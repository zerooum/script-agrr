mod app;
mod credentials;
mod discovery;
mod executor;
mod manifest;
mod runtime;
mod ui;

use std::io;
use std::path::Path;

use app::{App, Mode};
use executor::CollectedArgs;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> io::Result<()> {
    let scripts_dir = Path::new("scripts");

    let registry = discovery::discover(scripts_dir).await;

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Ensure we restore the terminal even if we panic
    let result = run_app(&mut terminal, registry.scripts, registry.warnings).await;

    // ── Terminal teardown ─────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

// Local tag to break the borrow cycle: determine mode variant (short immutable
// borrow, produces a Copy value), release the borrow, then call handler with
// full &mut App. This avoids borrow-checker conflicts when handlers need to
// call &mut self methods on App.
#[derive(Clone, Copy)]
enum Which {
    Menu, Search, Args, Cred, AskSave, Running,
    Result, AuthError, CredMgr, CredMgrSaving, CredMgrClear, Quit,
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    scripts: Vec<discovery::ScriptEntry>,
    warnings: Vec<discovery::LoadWarning>,
) -> io::Result<()> {
    let mut app = App::new(scripts, warnings);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        // Step 1: determine which handler (immutable borrow, then released)
        let which = match &app.mode {
            Mode::Menu                      => Which::Menu,
            Mode::Search                    => Which::Search,
            Mode::CollectingArgs { .. }     => Which::Args,
            Mode::CollectingCred { .. }     => Which::Cred,
            Mode::AskSaveCred { .. }        => Which::AskSave,
            Mode::Running                   => Which::Running,
            Mode::ExecutionResult { .. }    => Which::Result,
            Mode::AuthErrorPrompt { .. }    => Which::AuthError,
            Mode::CredManager { .. }        => Which::CredMgr,
            Mode::CredManagerSaving { .. }  => Which::CredMgrSaving,
            Mode::CredManagerClearConfirm { .. } => Which::CredMgrClear,
            Mode::Quit                      => Which::Quit,
        };

        // Step 2: call handler with full &mut App (borrow released above)
        let quit = match which {
            Which::Menu        => handle_menu(&mut app, key),
            Which::Search      => handle_search(&mut app, key),
            Which::Args        => { handle_collecting_args(&mut app, key); false }
            Which::Cred        => { handle_collecting_cred(&mut app, key); false }
            Which::AskSave     => { handle_ask_save_cred(&mut app, key); false }
            Which::Running     => false,
            Which::Result      => { app.return_to_menu(); false }
            Which::AuthError   => { handle_auth_error(&mut app, key); false }
            Which::CredMgr     => handle_cred_manager(&mut app, key),
            Which::CredMgrSaving => { handle_cred_manager_saving(&mut app, key); false }
            Which::CredMgrClear  => { handle_cred_manager_clear(&mut app, key); false }
            Which::Quit        => true,
        };
        if quit { break; }
    }

    Ok(())
}

// ── Menu navigation ───────────────────────────────────────────────────────────

fn handle_menu(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    // Guard: any key dismisses the warnings panel
    if !app.warnings_dismissed && !app.warnings.is_empty() {
        app.warnings_dismissed = true;
        return false;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Char('/') => app.mode = Mode::Search,
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if app.registry.iter().any(|e| !e.manifest.requires_auth.is_empty()) {
                app.mode = Mode::CredManager { cursor: 0 };
            }
        }
        KeyCode::Enter => app.begin_execute(),
        _ => {}
    }
    false
}

// ── Search / fuzzy filter ─────────────────────────────────────────────────────

fn handle_search(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Menu;
            app.clear_search();
        }
        KeyCode::Enter => {
            app.mode = Mode::Menu;
            app.begin_execute();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),
        KeyCode::Char(c) => {
            let mut q = app.search_query.clone();
            q.push(c);
            app.update_search(&q);
        }
        KeyCode::Backspace => {
            let mut q = app.search_query.clone();
            q.pop();
            app.update_search(&q);
        }
        _ => {}
    }
    false
}

// ── Arg collection ────────────────────────────────────────────────────────────

fn handle_collecting_args(app: &mut App, key: crossterm::event::KeyEvent) {
    // Extract needed values in a short-lived borrow (released before app method calls)
    let (script_idx, arg_idx_val, arg_name, arg_options) = {
        let Mode::CollectingArgs { script_idx, arg_idx, .. } = &app.mode else { return; };
        let s = *script_idx;
        let a = *arg_idx;
        let name = app.registry[s].manifest.args[a].name.clone();
        let opts = app.registry[s].manifest.args[a].options.clone();
        (s, a, name, opts)
    };

    match key.code {
        KeyCode::Esc => app.return_to_menu(),
        KeyCode::Enter => {
            let value = {
                let Mode::CollectingArgs { collected, .. } = &app.mode else { return; };
                collected.get(&arg_name).cloned().unwrap_or_default()
            };
            // Validate against options if constrained
            if !arg_options.is_empty() && !arg_options.iter().any(|o| o == &value) {
                return; // Invalid option — don't advance
            }
            let (new_collected, new_pending) = {
                let Mode::CollectingArgs { collected, pending_creds, .. } = &app.mode else { return; };
                let mut nc = collected.clone();
                nc.insert(arg_name.clone(), value);
                (nc, pending_creds.clone())
            };
            app.start_arg_or_cred_collection(script_idx, arg_idx_val + 1, new_collected, new_pending);
        }
        KeyCode::Backspace => {
            if let Mode::CollectingArgs { collected, .. } = &mut app.mode {
                if let Some(v) = collected.get_mut(&arg_name) {
                    v.pop();
                }
            }
        }
        KeyCode::Char(c) => {
            if let Mode::CollectingArgs { collected, .. } = &mut app.mode {
                collected.entry(arg_name).or_default().push(c);
            }
        }
        _ => {}
    }
}

// ── Credential collection ─────────────────────────────────────────────────────

fn handle_collecting_cred(app: &mut App, key: crossterm::event::KeyEvent) {
    let (script_idx, cred_key_str, resume) = {
        let Mode::CollectingCred { script_idx, key: cred_key, resume_arg_idx, .. } = &app.mode else { return; };
        (*script_idx, cred_key.clone(), *resume_arg_idx)
    };

    // Input is tracked in pending_creds under a __input__<key> sentinel
    match key.code {
        KeyCode::Esc => app.return_to_menu(),
        KeyCode::Enter => {
            let (value, new_args, new_pending) = {
                let Mode::CollectingCred { collected_args, pending_creds, .. } = &app.mode else { return; };
                let input_key = format!("__input__{}", cred_key_str);
                let value = pending_creds.get(&input_key).cloned().unwrap_or_default();
                let mut np = pending_creds.clone();
                np.remove(&input_key);
                np.insert(cred_key_str.clone(), value.clone());
                (value, collected_args.clone(), np)
            };
            app.mode = Mode::AskSaveCred {
                script_idx,
                key: cred_key_str,
                value,
                resume_arg_idx: resume,
                collected_args: new_args,
                pending_creds: new_pending,
            };
        }
        KeyCode::Backspace => {
            let input_key = format!("__input__{}", cred_key_str);
            if let Mode::CollectingCred { pending_creds, .. } = &mut app.mode {
                if let Some(v) = pending_creds.get_mut(&input_key) {
                    v.pop();
                }
            }
        }
        KeyCode::Char(c) => {
            let input_key = format!("__input__{}", cred_key_str);
            if let Mode::CollectingCred { pending_creds, .. } = &mut app.mode {
                pending_creds.entry(input_key).or_default().push(c);
            }
        }
        _ => {}
    }
}

// ── Ask to save credential ────────────────────────────────────────────────────

fn handle_ask_save_cred(app: &mut App, key: crossterm::event::KeyEvent) {
    let (script_idx, cred_key, value, resume, new_args, new_pending) = {
        let Mode::AskSaveCred {
            script_idx, key: k, value, resume_arg_idx, collected_args, pending_creds,
        } = &app.mode else { return; };
        (*script_idx, k.clone(), value.clone(), *resume_arg_idx,
         collected_args.clone(), pending_creds.clone())
    };

    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
            // Save to keychain
            let _ = credentials::set(&cred_key, &value);
            let mut np = new_pending;
            np.remove(&cred_key); // remove session-only copy since it's now persisted
            app.start_arg_or_cred_collection(script_idx, resume, new_args, np);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            // Don't save — keep in pending_creds for this session only
            app.start_arg_or_cred_collection(script_idx, resume, new_args, new_pending);
        }
        _ => {}
    }
}

// ── Auth error prompt ─────────────────────────────────────────────────────────

fn handle_auth_error(app: &mut App, key: crossterm::event::KeyEvent) {
    let Mode::AuthErrorPrompt { script_idx } = &app.mode else { return; };
    let idx = *script_idx;
    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
            // Retry: restart the whole collection flow
            app.start_arg_or_cred_collection(
                idx, 0, CollectedArgs::new(), std::collections::HashMap::new(),
            );
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.return_to_menu(),
        _ => {}
    }
}

// ── Credential manager ────────────────────────────────────────────────────────

fn handle_cred_manager(app: &mut App, key: crossterm::event::KeyEvent) -> bool {
    let cur = match &app.mode {
        Mode::CredManager { cursor } => *cursor,
        _ => return false,
    };

    let scripts_with_auth: Vec<usize> = (0..app.registry.len())
        .filter(|&i| !app.registry[i].manifest.requires_auth.is_empty())
        .collect();
    let max_cursor = scripts_with_auth.len(); // cursor 0 = Globais, 1+ = scripts

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Menu,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Up | KeyCode::Char('k') => {
            if cur > 0 {
                app.mode = Mode::CredManager { cursor: cur - 1 };
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if cur < max_cursor {
                app.mode = Mode::CredManager { cursor: cur + 1 };
            }
        }
        KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
            if cur == 0 {
                let global_keys = credentials::GLOBAL_KEYS;
                let all_saved = global_keys.iter().all(|&k| credentials::get(k).is_some());
                if all_saved {
                    app.mode = Mode::CredManagerClearConfirm { cred_manager_cursor: cur, script_idx: None };
                } else if let Some(k) = global_keys.iter().find(|&&k| credentials::get(k).is_none()).map(|&k| k.to_string()) {
                    app.mode = Mode::CredManagerSaving { cred_manager_cursor: cur, script_idx: None, key: k, input: String::new() };
                }
            } else if let Some(&script_idx) = scripts_with_auth.get(cur - 1) {
                let requires_auth = app.registry[script_idx].manifest.requires_auth.clone();
                let all_saved = requires_auth.iter().all(|k| credentials::get(k).is_some());
                if all_saved {
                    app.mode = Mode::CredManagerClearConfirm { cred_manager_cursor: cur, script_idx: Some(script_idx) };
                } else if let Some(k) = requires_auth.into_iter().find(|k| credentials::get(k).is_none()) {
                    app.mode = Mode::CredManagerSaving { cred_manager_cursor: cur, script_idx: Some(script_idx), key: k, input: String::new() };
                }
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            if cur == 0 {
                if credentials::GLOBAL_KEYS.iter().any(|&k| credentials::get(k).is_some()) {
                    app.mode = Mode::CredManagerClearConfirm { cred_manager_cursor: cur, script_idx: None };
                }
            } else if let Some(&script_idx) = scripts_with_auth.get(cur - 1) {
                if app.registry[script_idx].manifest.requires_auth.iter().any(|k| credentials::get(k).is_some()) {
                    app.mode = Mode::CredManagerClearConfirm { cred_manager_cursor: cur, script_idx: Some(script_idx) };
                }
            }
        }
        _ => {}
    }
    false
}

// ── Credential manager — saving ───────────────────────────────────────────────

fn handle_cred_manager_saving(app: &mut App, key: crossterm::event::KeyEvent) {
    let (cursor_bk, sidx) = {
        let Mode::CredManagerSaving { cred_manager_cursor, script_idx, .. } = &app.mode else { return; };
        (*cred_manager_cursor, *script_idx)
    };

    match key.code {
        KeyCode::Esc => app.mode = Mode::CredManager { cursor: cursor_bk },
        KeyCode::Backspace => {
            if let Mode::CredManagerSaving { input, .. } = &mut app.mode {
                input.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::CredManagerSaving { input, .. } = &mut app.mode {
                input.push(c);
            }
        }
        KeyCode::Enter => {
            let (value, saved_key) = {
                let Mode::CredManagerSaving { input, key: k, .. } = &app.mode else { return; };
                (input.clone(), k.clone())
            };
            let _ = credentials::set(&saved_key, &value);
            let next_key = match sidx {
                None => credentials::GLOBAL_KEYS.iter()
                    .find(|&&k| credentials::get(k).is_none())
                    .map(|&k| k.to_string()),
                Some(idx) => app.registry[idx].manifest.requires_auth.clone()
                    .into_iter().find(|k| credentials::get(k).is_none()),
            };
            app.mode = match next_key {
                Some(k) => Mode::CredManagerSaving {
                    cred_manager_cursor: cursor_bk,
                    script_idx: sidx,
                    key: k,
                    input: String::new(),
                },
                None => Mode::CredManager { cursor: cursor_bk },
            };
        }
        _ => {}
    }
}

// ── Credential manager — clear confirm ───────────────────────────────────────

fn handle_cred_manager_clear(app: &mut App, key: crossterm::event::KeyEvent) {
    let (cursor_bk, sidx) = {
        let Mode::CredManagerClearConfirm { cred_manager_cursor, script_idx } = &app.mode else { return; };
        (*cred_manager_cursor, *script_idx)
    };

    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
            let keys_to_clear: Vec<String> = match sidx {
                None => credentials::GLOBAL_KEYS.iter().map(|k| k.to_string()).collect(),
                Some(idx) => app.registry[idx].manifest.requires_auth.clone(),
            };
            credentials::delete_all(&keys_to_clear);
            app.mode = Mode::CredManager { cursor: cursor_bk };
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = Mode::CredManager { cursor: cursor_bk };
        }
        _ => {}
    }
}
