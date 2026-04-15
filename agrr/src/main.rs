use std::io;
use std::path::PathBuf;

use agrr::app::{App, Mode};
use agrr::executor::CollectedArgs;
use agrr::{credentials, discovery};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Resolve the scripts directory.
///
/// Priority:
/// 1. `scripts/` next to the running executable (distribution / `build/` layout)
/// 2. `scripts/` relative to the current working directory (development mode)
fn resolve_scripts_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("scripts");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    PathBuf::from("scripts")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let scripts_dir = resolve_scripts_dir();
    let scripts_dir = scripts_dir.as_path();

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
    Menu, Search, SelectingSubcmd, Args, Cred, AskSave, Running,
    Result, AuthError, CredMgr, CredMgrSaving, CredMgrClear, Quit,
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    scripts: Vec<discovery::ScriptEntry>,
    warnings: Vec<discovery::LoadWarning>,
) -> io::Result<()> {
    let mut app = App::new(scripts, warnings);

    loop {
        terminal.draw(|f| agrr::render(f, &app))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        // Step 1: determine which handler (immutable borrow, then released)
        let which = match &app.mode {
            Mode::Menu                          => Which::Menu,
            Mode::Search                        => Which::Search,
            Mode::SelectingSubcommand { .. }    => Which::SelectingSubcmd,
            Mode::CollectingArgs { .. }         => Which::Args,
            Mode::CollectingCred { .. }         => Which::Cred,
            Mode::AskSaveCred { .. }            => Which::AskSave,
            Mode::Running                       => Which::Running,
            Mode::ExecutionResult { .. }        => Which::Result,
            Mode::AuthErrorPrompt { .. }        => Which::AuthError,
            Mode::CredManager { .. }            => Which::CredMgr,
            Mode::CredManagerSaving { .. }      => Which::CredMgrSaving,
            Mode::CredManagerClearConfirm { .. } => Which::CredMgrClear,
            Mode::Quit                          => Which::Quit,
        };

        // Step 2: call handler with full &mut App (borrow released above)
        let quit = match which {
            Which::Menu           => handle_menu(&mut app, key),
            Which::Search         => handle_search(&mut app, key),
            Which::SelectingSubcmd => { handle_selecting_subcommand(&mut app, key); false }
            Which::Args           => { handle_collecting_args(&mut app, key); false }
            Which::Cred           => { handle_collecting_cred(&mut app, key); false }
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
    use agrr::manifest::{ArgType, Pattern};

    // Extract all needed data in a short-lived borrow
    let (script_idx, arg_idx_val, arg_name, arg_type, arg_options, arg_max_length,
         arg_pattern, arg_required, arg_default, current_cursor, current_ms,
         selected_subcommand) = {
        let Mode::CollectingArgs {
            script_idx, arg_idx, select_cursor, multiselect_selected,
            selected_subcommand, ..
        } = &app.mode else { return; };
        let s = *script_idx;
        let a = *arg_idx;
        let sc = *select_cursor;
        let ms = multiselect_selected.clone();
        let subcmd = selected_subcommand.clone();
        let arg = app.registry[s].manifest.effective_args(subcmd.as_deref())[a].clone();
        (s, a, arg.name, arg.arg_type,
         arg.options, arg.max_length, arg.pattern,
         arg.required, arg.default, sc, ms, subcmd)
    };

    match arg_type {
        ArgType::Text => match key.code {
            KeyCode::Esc => app.return_to_menu(),
            KeyCode::Enter => {
                let value = {
                    let Mode::CollectingArgs { collected, .. } = &app.mode else { return; };
                    collected.get(&arg_name).cloned().unwrap_or_default()
                };
                // Apply default when blank
                let final_value = if value.is_empty() {
                    arg_default.unwrap_or_default()
                } else {
                    value
                };
                // Enforce required
                if final_value.is_empty() && arg_required {
                    if let Mode::CollectingArgs { validation_error, .. } = &mut app.mode {
                        *validation_error = Some("campo obrigatório".to_string());
                    }
                    return;
                }
                let (new_collected, new_pending) = {
                    let Mode::CollectingArgs { collected, pending_creds, .. } = &app.mode else { return; };
                    let mut nc = collected.clone();
                    nc.insert(arg_name.clone(), final_value);
                    (nc, pending_creds.clone())
                };
                app.start_arg_or_cred_collection(script_idx, arg_idx_val + 1, new_collected, new_pending, selected_subcommand);
            }
            KeyCode::Backspace => {
                if let Mode::CollectingArgs { collected, validation_error, .. } = &mut app.mode {
                    *validation_error = None;
                    if let Some(v) = collected.get_mut(&arg_name) {
                        v.pop();
                    }
                }
            }
            KeyCode::Char(c) => {
                // Filter by pattern
                let allowed = match &arg_pattern {
                    Some(Pattern::Numeric) => c.is_ascii_digit(),
                    Some(Pattern::Alpha) => c.is_alphabetic(),
                    Some(Pattern::Alphanumeric) => c.is_alphanumeric(),
                    None => true,
                };
                if !allowed {
                    return;
                }
                if let Mode::CollectingArgs { collected, validation_error, .. } = &mut app.mode {
                    let current = collected.entry(arg_name.clone()).or_default();
                    // Enforce max_length
                    if let Some(max) = arg_max_length {
                        if current.chars().count() >= max as usize {
                            *validation_error = Some(format!("máximo de {} caracteres", max));
                            return;
                        }
                    }
                    *validation_error = None;
                    current.push(c);
                }
            }
            _ => {}
        },

        ArgType::Select => match key.code {
            KeyCode::Esc => app.return_to_menu(),
            KeyCode::Up | KeyCode::Char('k') => {
                if let Mode::CollectingArgs { select_cursor, validation_error, .. } = &mut app.mode {
                    *validation_error = None;
                    if *select_cursor > 0 {
                        *select_cursor -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Mode::CollectingArgs { select_cursor, validation_error, .. } = &mut app.mode {
                    *validation_error = None;
                    if *select_cursor + 1 < arg_options.len() {
                        *select_cursor += 1;
                    }
                }
            }
            KeyCode::Enter => {
                let selected = arg_options.get(current_cursor).cloned().unwrap_or_default();
                let (new_collected, new_pending) = {
                    let Mode::CollectingArgs { collected, pending_creds, .. } = &app.mode else { return; };
                    let mut nc = collected.clone();
                    nc.insert(arg_name.clone(), selected);
                    (nc, pending_creds.clone())
                };
                app.start_arg_or_cred_collection(script_idx, arg_idx_val + 1, new_collected, new_pending, selected_subcommand);
            }
            _ => {}
        },

        ArgType::MultiSelect => match key.code {
            KeyCode::Esc => app.return_to_menu(),
            KeyCode::Up | KeyCode::Char('k') => {
                if let Mode::CollectingArgs { select_cursor, validation_error, .. } = &mut app.mode {
                    *validation_error = None;
                    if *select_cursor > 0 {
                        *select_cursor -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Mode::CollectingArgs { select_cursor, validation_error, .. } = &mut app.mode {
                    *validation_error = None;
                    if *select_cursor + 1 < arg_options.len() {
                        *select_cursor += 1;
                    }
                }
            }
            KeyCode::Char(' ') => {
                if let Some(opt) = arg_options.get(current_cursor) {
                    let opt_str = opt.clone();
                    if let Mode::CollectingArgs { multiselect_selected, validation_error, .. } = &mut app.mode {
                        *validation_error = None;
                        if let Some(pos) = multiselect_selected.iter().position(|s| s == &opt_str) {
                            multiselect_selected.remove(pos);
                        } else {
                            multiselect_selected.push(opt_str);
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if arg_required && current_ms.is_empty() {
                    if let Mode::CollectingArgs { validation_error, .. } = &mut app.mode {
                        *validation_error = Some("selecione ao menos uma opção".to_string());
                    }
                    return;
                }
                let joined = current_ms.join(",");
                let (new_collected, new_pending) = {
                    let Mode::CollectingArgs { collected, pending_creds, .. } = &app.mode else { return; };
                    let mut nc = collected.clone();
                    nc.insert(arg_name.clone(), joined);
                    (nc, pending_creds.clone())
                };
                app.start_arg_or_cred_collection(script_idx, arg_idx_val + 1, new_collected, new_pending, selected_subcommand);
            }
            _ => {}
        },
    }
}

// ── Credential collection ─────────────────────────────────────────────────────

fn handle_collecting_cred(app: &mut App, key: crossterm::event::KeyEvent) {
    let (script_idx, cred_key_str, resume, selected_subcommand) = {
        let Mode::CollectingCred { script_idx, key: cred_key, resume_arg_idx, selected_subcommand, .. } = &app.mode else { return; };
        (*script_idx, cred_key.clone(), *resume_arg_idx, selected_subcommand.clone())
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
                selected_subcommand,
            };
        }
        KeyCode::Backspace => {
            let input_key = format!("__input__{}", cred_key_str);
            if let Mode::CollectingCred { pending_creds, validation_error, .. } = &mut app.mode {
                *validation_error = None;
                if let Some(v) = pending_creds.get_mut(&input_key) {
                    v.pop();
                }
            }
        }
        KeyCode::Char(c) => {
            use agrr::manifest::Pattern;
            let input_key = format!("__input__{}", cred_key_str);
            let constraint = agrr::credentials::global_cred_constraint(&cred_key_str);

            // Check pattern constraint
            if let Some(ref con) = constraint {
                let allowed = match &con.pattern {
                    Some(Pattern::Numeric) => c.is_ascii_digit(),
                    Some(Pattern::Alpha) => c.is_alphabetic(),
                    Some(Pattern::Alphanumeric) => c.is_alphanumeric(),
                    None => true,
                };
                if !allowed {
                    if let Mode::CollectingCred { validation_error, .. } = &mut app.mode {
                        *validation_error = Some("apenas dígitos permitidos".to_string());
                    }
                    return;
                }
            }

            if let Mode::CollectingCred { pending_creds, validation_error, .. } = &mut app.mode {
                let current = pending_creds.entry(input_key).or_default();
                // Check max_length constraint
                if let Some(ref con) = constraint {
                    if current.chars().count() >= con.max_length as usize {
                        *validation_error = Some(format!("máximo de {} caracteres", con.max_length));
                        return;
                    }
                }
                *validation_error = None;
                current.push(c);
            }
        }
        _ => {}
    }
}

// ── Ask to save credential ────────────────────────────────────────────────────

fn handle_ask_save_cred(app: &mut App, key: crossterm::event::KeyEvent) {
    let (script_idx, cred_key, value, resume, new_args, new_pending, selected_subcommand) = {
        let Mode::AskSaveCred {
            script_idx, key: k, value, resume_arg_idx, collected_args, pending_creds,
            selected_subcommand,
        } = &app.mode else { return; };
        (*script_idx, k.clone(), value.clone(), *resume_arg_idx,
         collected_args.clone(), pending_creds.clone(), selected_subcommand.clone())
    };

    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
            // Save to keychain
            let _ = credentials::set(&cred_key, &value);
            let mut np = new_pending;
            np.remove(&cred_key); // remove session-only copy since it's now persisted
            app.start_arg_or_cred_collection(script_idx, resume, new_args, np, selected_subcommand);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            // Don't save — keep in pending_creds for this session only
            app.start_arg_or_cred_collection(script_idx, resume, new_args, new_pending, selected_subcommand);
        }
        _ => {}
    }
}

// ── Subcommand selection ──────────────────────────────────────────────────────

fn handle_selecting_subcommand(app: &mut App, key: crossterm::event::KeyEvent) {
    let (script_idx, cursor, pending_creds) = {
        let Mode::SelectingSubcommand { script_idx, cursor, pending_creds } = &app.mode else { return; };
        (*script_idx, *cursor, pending_creds.clone())
    };
    let subcommand_count = app.registry[script_idx].manifest.subcommands.len();
    match key.code {
        KeyCode::Esc => app.return_to_menu(),
        KeyCode::Up | KeyCode::Char('k') => {
            if cursor > 0 {
                if let Mode::SelectingSubcommand { cursor: c, .. } = &mut app.mode {
                    *c -= 1;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if cursor + 1 < subcommand_count {
                if let Mode::SelectingSubcommand { cursor: c, .. } = &mut app.mode {
                    *c += 1;
                }
            }
        }
        KeyCode::Enter => {
            let subcommand_name = app.registry[script_idx].manifest.subcommands[cursor].name.clone();
            app.start_arg_or_cred_collection(
                script_idx, 0, CollectedArgs::new(), pending_creds, Some(subcommand_name),
            );
        }
        _ => {}
    }
}

// ── Auth error prompt ─────────────────────────────────────────────────────────

fn handle_auth_error(app: &mut App, key: crossterm::event::KeyEvent) {
    let (idx, selected_subcommand) = {
        let Mode::AuthErrorPrompt { script_idx, selected_subcommand } = &app.mode else { return; };
        (*script_idx, selected_subcommand.clone())
    };
    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
            // Retry: restart collection but preserve subcommand selection
            app.start_arg_or_cred_collection(
                idx, 0, CollectedArgs::new(), std::collections::HashMap::new(), selected_subcommand,
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
            use agrr::manifest::Pattern;
            // For global creds (script_idx == None), enforce hardcoded constraints
            if sidx.is_none() {
                let key_name = {
                    let Mode::CredManagerSaving { key: k, .. } = &app.mode else { return; };
                    k.clone()
                };
                let constraint = agrr::credentials::global_cred_constraint(&key_name);
                if let Some(ref con) = constraint {
                    let allowed = match &con.pattern {
                        Some(Pattern::Numeric) => c.is_ascii_digit(),
                        Some(Pattern::Alpha) => c.is_alphabetic(),
                        Some(Pattern::Alphanumeric) => c.is_alphanumeric(),
                        None => true,
                    };
                    if !allowed {
                        return;
                    }
                    if let Mode::CredManagerSaving { input, .. } = &mut app.mode {
                        if input.chars().count() >= con.max_length as usize {
                            return;
                        }
                    }
                }
            }
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use agrr::app::{App, Mode};
    use agrr::discovery::ScriptEntry;
    use agrr::manifest::ScriptManifest;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn resolve_scripts_dir_fallback_to_cwd() {
        let result = resolve_scripts_dir();
        assert!(result.ends_with("scripts"));
    }

    fn make_global_auth_entry() -> ScriptEntry {
        ScriptEntry {
            path: PathBuf::from("/tmp/test.py"),
            manifest: ScriptManifest {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: "test".to_string(),
                group: "test".to_string(),
                args: vec![],
                requires_auth: vec![],
                runtime: None,
                global_auth: true,
                subcommands: vec![],
            },
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app_collecting_senha() -> App {
        let mut app = App::new(vec![make_global_auth_entry()], vec![]);
        app.begin_execute();
        // After begin_execute with global_auth, should be in CollectingCred for CHAVE
        // Simulate CHAVE already collected, move to SENHA
        // Set mode directly to SENHA collection
        app.mode = Mode::CollectingCred {
            script_idx: 0,
            key: "SENHA".to_string(),
            resume_arg_idx: 0,
            collected_args: agrr::executor::CollectedArgs::new(),
            pending_creds: std::collections::HashMap::new(),
            selected_subcommand: None,
            validation_error: None,
        };
        app
    }

    fn app_collecting_chave() -> App {
        let mut app = App::new(vec![make_global_auth_entry()], vec![]);
        app.mode = Mode::CollectingCred {
            script_idx: 0,
            key: "CHAVE".to_string(),
            resume_arg_idx: 0,
            collected_args: agrr::executor::CollectedArgs::new(),
            pending_creds: std::collections::HashMap::new(),
            selected_subcommand: None,
            validation_error: None,
        };
        app
    }

    #[test]
    fn senha_rejects_non_digit_keystroke_and_sets_validation_error() {
        let mut app = app_collecting_senha();
        handle_collecting_cred(&mut app, key(KeyCode::Char('a')));
        let Mode::CollectingCred { validation_error, pending_creds, .. } = &app.mode else {
            panic!("expected CollectingCred");
        };
        assert!(validation_error.is_some(), "validation_error should be set");
        // The char should NOT have been appended
        assert!(pending_creds.get("__input__SENHA").map_or(true, |v| v.is_empty()));
    }

    #[test]
    fn senha_accepts_digit_keystroke() {
        let mut app = app_collecting_senha();
        handle_collecting_cred(&mut app, key(KeyCode::Char('5')));
        let Mode::CollectingCred { validation_error, pending_creds, .. } = &app.mode else {
            panic!("expected CollectingCred");
        };
        assert!(validation_error.is_none(), "should have no error after valid digit");
        assert_eq!(pending_creds.get("__input__SENHA").map(String::as_str), Some("5"));
    }

    #[test]
    fn senha_rejects_ninth_character_exceeding_max_length() {
        let mut app = app_collecting_senha();
        // Type 8 valid digits
        for _ in 0..8 {
            handle_collecting_cred(&mut app, key(KeyCode::Char('1')));
        }
        // 9th digit must be rejected
        handle_collecting_cred(&mut app, key(KeyCode::Char('2')));
        let Mode::CollectingCred { validation_error, pending_creds, .. } = &app.mode else {
            panic!("expected CollectingCred");
        };
        assert!(validation_error.is_some(), "validation_error should be set for exceeding max_length");
        let input = pending_creds.get("__input__SENHA").map(String::as_str).unwrap_or("");
        assert_eq!(input.len(), 8, "input should remain at 8 characters");
    }

    #[test]
    fn chave_rejects_ninth_character_exceeding_max_length() {
        let mut app = app_collecting_chave();
        // Type 8 characters
        for _ in 0..8 {
            handle_collecting_cred(&mut app, key(KeyCode::Char('x')));
        }
        // 9th must be rejected
        handle_collecting_cred(&mut app, key(KeyCode::Char('y')));
        let Mode::CollectingCred { validation_error, pending_creds, .. } = &app.mode else {
            panic!("expected CollectingCred");
        };
        assert!(validation_error.is_some(), "validation_error should be set for exceeding max_length");
        let input = pending_creds.get("__input__CHAVE").map(String::as_str).unwrap_or("");
        assert_eq!(input.len(), 8, "CHAVE input should remain at 8 characters");
    }

    #[test]
    fn chave_accepts_any_char_within_limit() {
        let mut app = app_collecting_chave();
        handle_collecting_cred(&mut app, key(KeyCode::Char('a')));
        let Mode::CollectingCred { validation_error, .. } = &app.mode else {
            panic!("expected CollectingCred");
        };
        assert!(validation_error.is_none());
    }

    #[test]
    fn backspace_clears_validation_error() {
        let mut app = app_collecting_senha();
        // Trigger a validation error
        handle_collecting_cred(&mut app, key(KeyCode::Char('a')));
        assert!(matches!(&app.mode, Mode::CollectingCred { validation_error: Some(_), .. }));
        // Backspace clears it
        handle_collecting_cred(&mut app, key(KeyCode::Backspace));
        assert!(matches!(&app.mode, Mode::CollectingCred { validation_error: None, .. }));
    }
}
