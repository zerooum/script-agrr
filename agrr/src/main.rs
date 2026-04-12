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

        match &mut app.mode {
            // ── Warnings dismissal ──────────────────────────────────────────
            Mode::Menu if !app.warnings_dismissed && !app.warnings.is_empty() => {
                app.warnings_dismissed = true;
            }

            // ── Menu navigation ─────────────────────────────────────────────
            Mode::Menu => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Char('/') => {
                    app.mode = Mode::Search;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    let has_auth = app.registry.iter().any(|e| !e.manifest.requires_auth.is_empty());
                    if has_auth {
                        app.mode = Mode::CredManager { cursor: 0 };
                    }
                }
                KeyCode::Enter => {
                    app.begin_execute();
                }
                _ => {}
            },

            // ── Search / fuzzy filter ────────────────────────────────────────
            Mode::Search => match key.code {
                KeyCode::Esc => {
                    app.mode = Mode::Menu;
                    app.clear_search();
                }
                KeyCode::Enter => {
                    app.mode = Mode::Menu;
                    app.begin_execute();
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
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
            },

            // ── Arg collection ───────────────────────────────────────────────
            Mode::CollectingArgs {
                script_idx,
                arg_idx,
                collected,
                pending_creds,
            } => {
                let script_idx = *script_idx;
                let arg_idx_val = *arg_idx;
                let arg_name = app.registry[script_idx].manifest.args[arg_idx_val]
                    .name
                    .clone();
                let arg_options = app.registry[script_idx].manifest.args[arg_idx_val]
                    .options
                    .clone();

                match key.code {
                    KeyCode::Esc => app.return_to_menu(),
                    KeyCode::Enter => {
                        let value = collected
                            .get(&arg_name)
                            .cloned()
                            .unwrap_or_default();

                        // Validate against options if constrained
                        if !arg_options.is_empty()
                            && !arg_options.iter().any(|o| o == &value)
                        {
                            // Invalid option — don't advance
                        } else {
                            let mut new_collected = collected.clone();
                            new_collected.insert(arg_name, value);
                            let new_pending = pending_creds.clone();
                            app.start_arg_or_cred_collection(
                                script_idx,
                                arg_idx_val + 1,
                                new_collected,
                                new_pending,
                            );
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(v) = collected.get_mut(&arg_name) {
                            v.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        collected.entry(arg_name).or_default().push(c);
                    }
                    _ => {}
                }
            }

            // ── Credential collection ────────────────────────────────────────
            Mode::CollectingCred {
                script_idx,
                key: cred_key,
                resume_arg_idx,
                collected_args,
                pending_creds,
            } => {
                let script_idx = *script_idx;
                let cred_key_str = cred_key.clone();
                let resume = *resume_arg_idx;

                // We track input in a temporary string stored in pending_creds
                match key.code {
                    KeyCode::Esc => app.return_to_menu(),
                    KeyCode::Enter => {
                        let value = pending_creds
                            .get(&format!("__input__{}", cred_key_str))
                            .cloned()
                            .unwrap_or_default();
                        pending_creds.remove(&format!("__input__{}", cred_key_str));

                        let mut new_pending = pending_creds.clone();
                        new_pending.remove(&format!("__input__{}", cred_key_str));
                        new_pending.insert(cred_key_str.clone(), value.clone());

                        let new_args = collected_args.clone();

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
                        if let Some(v) = pending_creds.get_mut(&input_key) {
                            v.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        let input_key = format!("__input__{}", cred_key_str);
                        pending_creds.entry(input_key).or_default().push(c);
                    }
                    _ => {}
                }
            }

            // ── Ask to save credential ────────────────────────────────────────
            Mode::AskSaveCred {
                script_idx,
                key: cred_key,
                value,
                resume_arg_idx,
                collected_args,
                pending_creds,
            } => {
                let script_idx = *script_idx;
                let resume = *resume_arg_idx;

                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                        // Save to keychain
                        let _ = credentials::set(cred_key, value);
                        // Remove session-only copy since it's now persisted
                        pending_creds.remove(cred_key);
                        let new_args = collected_args.clone();
                        let new_pending = pending_creds.clone();
                        app.start_arg_or_cred_collection(script_idx, resume, new_args, new_pending);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Don't save — keep in pending_creds for this session only
                        let new_args = collected_args.clone();
                        let new_pending = pending_creds.clone();
                        app.start_arg_or_cred_collection(script_idx, resume, new_args, new_pending);
                    }
                    _ => {}
                }
            }

            // ── Execution result ─────────────────────────────────────────────
            Mode::ExecutionResult { exit_code: _, elapsed_ms: _ } => {
                app.return_to_menu();
            }

            // ── Auth error prompt ─────────────────────────────────────────────
            Mode::AuthErrorPrompt { script_idx } => {
                let idx = *script_idx;
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                        // Retry: restart the whole collection flow
                        app.start_arg_or_cred_collection(
                            idx,
                            0,
                            CollectedArgs::new(),
                            std::collections::HashMap::new(),
                        );
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.return_to_menu();
                    }
                    _ => {}
                }
            }

            Mode::Running => {
                // Running is synchronous — this branch is only reached if
                // we ever add async execution. No-op for now.
            }

            // ── Credential manager ───────────────────────────────────────────
            Mode::CredManager { cursor } => {
                let cur = *cursor;
                let scripts_with_auth: Vec<usize> = (0..app.registry.len())
                    .filter(|&i| !app.registry[i].manifest.requires_auth.is_empty())
                    .collect();
                // cursor 0 = Globais, cursor 1..=scripts_with_auth.len() = scripts
                let max_cursor = scripts_with_auth.len(); // inclusive (global at 0)

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Menu,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
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
                            // Global entry
                            let global_keys = credentials::GLOBAL_KEYS;
                            let all_saved = global_keys.iter().all(|&k| credentials::get(k).is_some());
                            if all_saved {
                                app.mode = Mode::CredManagerClearConfirm {
                                    cred_manager_cursor: cur,
                                    script_idx: None,
                                };
                            } else {
                                let next_key = global_keys.iter()
                                    .find(|&&k| credentials::get(k).is_none())
                                    .map(|&k| k.to_string());
                                if let Some(k) = next_key {
                                    app.mode = Mode::CredManagerSaving {
                                        cred_manager_cursor: cur,
                                        script_idx: None,
                                        key: k,
                                        input: String::new(),
                                    };
                                }
                            }
                        } else if let Some(&script_idx) = scripts_with_auth.get(cur - 1) {
                            let requires_auth = app.registry[script_idx].manifest.requires_auth.clone();
                            let all_saved = requires_auth.iter().all(|k| credentials::get(k).is_some());
                            if all_saved {
                                app.mode = Mode::CredManagerClearConfirm {
                                    cred_manager_cursor: cur,
                                    script_idx: Some(script_idx),
                                };
                            } else {
                                let next_key = requires_auth.into_iter()
                                    .find(|k| credentials::get(k).is_none());
                                if let Some(k) = next_key {
                                    app.mode = Mode::CredManagerSaving {
                                        cred_manager_cursor: cur,
                                        script_idx: Some(script_idx),
                                        key: k,
                                        input: String::new(),
                                    };
                                }
                            }
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if cur == 0 {
                            let has_any = credentials::GLOBAL_KEYS.iter()
                                .any(|&k| credentials::get(k).is_some());
                            if has_any {
                                app.mode = Mode::CredManagerClearConfirm {
                                    cred_manager_cursor: cur,
                                    script_idx: None,
                                };
                            }
                        } else if let Some(&script_idx) = scripts_with_auth.get(cur - 1) {
                            let has_any = app.registry[script_idx].manifest.requires_auth.iter()
                                .any(|k| credentials::get(k).is_some());
                            if has_any {
                                app.mode = Mode::CredManagerClearConfirm {
                                    cred_manager_cursor: cur,
                                    script_idx: Some(script_idx),
                                };
                            }
                        }
                    }
                    _ => {}
                }
            }

            // ── Credential manager — saving ──────────────────────────────────
            Mode::CredManagerSaving {
                cred_manager_cursor,
                script_idx,
                key: cred_key,
                input,
            } => {
                let cursor_bk = *cred_manager_cursor;
                let sidx = *script_idx;

                match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::CredManager { cursor: cursor_bk };
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Enter => {
                        let value = input.clone();
                        let saved_key = cred_key.clone();
                        let _ = credentials::set(&saved_key, &value);

                        let next_key = match sidx {
                            None => {
                                // Global keys — find next missing one
                                credentials::GLOBAL_KEYS.iter()
                                    .find(|&&k| credentials::get(k).is_none())
                                    .map(|&k| k.to_string())
                            }
                            Some(idx) => {
                                let requires_auth = app.registry[idx].manifest.requires_auth.clone();
                                requires_auth.into_iter().find(|k| credentials::get(k).is_none())
                            }
                        };

                        if let Some(k) = next_key {
                            app.mode = Mode::CredManagerSaving {
                                cred_manager_cursor: cursor_bk,
                                script_idx: sidx,
                                key: k,
                                input: String::new(),
                            };
                        } else {
                            app.mode = Mode::CredManager { cursor: cursor_bk };
                        }
                    }
                    _ => {}
                }
            }

            // ── Credential manager — clear confirm ───────────────────────────
            Mode::CredManagerClearConfirm {
                cred_manager_cursor,
                script_idx,
            } => {
                let cursor_bk = *cred_manager_cursor;
                let sidx = *script_idx;

                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                        let keys_to_clear: Vec<String> = match sidx {
                            None => credentials::GLOBAL_KEYS
                                .iter().map(|k| k.to_string()).collect(),
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

            Mode::Quit => break,
        }
    }

    Ok(())
}
