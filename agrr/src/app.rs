use std::collections::HashMap;
use std::time::Instant;

use crate::credentials;
use crate::discovery::{LoadWarning, ScriptEntry};
use crate::executor::{self, CollectedArgs, ExitStatus, OutputLine};

/// Top-level application state.
pub struct App {
    pub registry: Vec<ScriptEntry>,
    pub warnings: Vec<LoadWarning>,

    /// Flat sorted list of script indices used for navigation.
    pub visible: Vec<usize>,
    /// Currently highlighted index into `visible`.
    pub cursor: usize,

    pub mode: Mode,
    pub search_query: String,

    pub output_lines: Vec<StyledLine>,
    pub output_scroll: usize,

    /// Master password for fallback credential store (cached per session).
    #[allow(dead_code)]
    pub master_password: Option<String>,

    /// Dismiss state for the warnings panel.
    pub warnings_dismissed: bool,
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    /// Normal menu navigation.
    Menu,
    /// Fuzzy search input active.
    Search,
    /// Collecting args for the selected script before execution.
    CollectingArgs {
        script_idx: usize,
        arg_idx: usize,
        collected: CollectedArgs,
        /// Session-only creds not yet in keychain.
        pending_creds: HashMap<String, String>,
        /// Cursor for select/multiselect option navigation.
        select_cursor: usize,
        /// Currently toggled options for multiselect (before confirmation).
        multiselect_selected: Vec<String>,
        /// Inline validation error to display below the input.
        validation_error: Option<String>,
    },
    /// Collecting a credential value for a specific key.
    CollectingCred {
        script_idx: usize,
        key: String,
        /// After collecting this cred, resume arg collection.
        resume_arg_idx: usize,
        collected_args: CollectedArgs,
        pending_creds: HashMap<String, String>,
    },
    /// Asking whether to save a credential
    AskSaveCred {
        script_idx: usize,
        key: String,
        value: String,
        resume_arg_idx: usize,
        collected_args: CollectedArgs,
        pending_creds: HashMap<String, String>,
    },
    /// Script running — output is streaming in.
    Running,
    /// Execution finished — showing result before returning to menu.
    ExecutionResult { exit_code: i32, elapsed_ms: u64 },
    /// Auth error — asking whether to retry.
    AuthErrorPrompt { script_idx: usize },
    /// Credential management screen.
    CredManager {
        cursor: usize,
    },
    /// Saving a credential from the credential manager.
    CredManagerSaving {
        cred_manager_cursor: usize,
        /// `None` means saving the agrr global credentials (CHAVE/SENHA).
        script_idx: Option<usize>,
        key: String,
        input: String,
    },
    /// Confirming credential deletion from the credential manager.
    CredManagerClearConfirm {
        cred_manager_cursor: usize,
        /// `None` means clearing the agrr global credentials (CHAVE/SENHA).
        script_idx: Option<usize>,
    },
    /// Quitting.
    #[allow(dead_code)]
    Quit,
}

#[derive(Debug, Clone)]
pub struct StyledLine {
    pub text: String,
    pub is_error: bool,
}

impl App {
    pub fn new(registry: Vec<ScriptEntry>, warnings: Vec<LoadWarning>) -> Self {
        let count = registry.len();
        let visible: Vec<usize> = (0..count).collect();
        Self {
            registry,
            warnings,
            visible,
            cursor: 0,
            mode: Mode::Menu,
            search_query: String::new(),
            output_lines: Vec::new(),
            output_scroll: 0,
            master_password: None,
            warnings_dismissed: false,
        }
    }

    // ─── Navigation ──────────────────────────────────────────────────────────

    pub fn move_up(&mut self) {
        if !self.visible.is_empty() && self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.visible.len() {
            self.cursor += 1;
        }
    }

    #[allow(dead_code)]
    pub fn selected_script(&self) -> Option<&ScriptEntry> {
        let idx = self.visible.get(self.cursor)?;
        self.registry.get(*idx)
    }

    pub fn selected_script_idx(&self) -> Option<usize> {
        self.visible.get(self.cursor).copied()
    }

    // ─── Search ──────────────────────────────────────────────────────────────

    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.apply_filter();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.visible = if q.is_empty() {
            (0..self.registry.len()).collect()
        } else {
            (0..self.registry.len())
                .filter(|&i| {
                    let s = &self.registry[i].manifest;
                    s.name.to_lowercase().contains(&q)
                        || s.description.to_lowercase().contains(&q)
                        || s.group.to_lowercase().contains(&q)
                })
                .collect()
        };
        // Clamp cursor
        if self.cursor >= self.visible.len() && !self.visible.is_empty() {
            self.cursor = self.visible.len() - 1;
        } else if self.visible.is_empty() {
            self.cursor = 0;
        }
    }

    // ─── Execution entry point ────────────────────────────────────────────────

    /// Start the flow for executing the currently selected script.
    /// This may transition to arg collection, cred collection, or directly
    /// to running if no args/creds are needed.
    pub fn begin_execute(&mut self) {
        let Some(idx) = self.selected_script_idx() else {
            return;
        };
        self.output_lines.clear();
        self.output_scroll = 0;
        self.start_arg_or_cred_collection(idx, 0, CollectedArgs::new(), HashMap::new());
    }

    /// Determine next step: collect next missing cred, next arg, or execute.
    pub fn start_arg_or_cred_collection(
        &mut self,
        script_idx: usize,
        arg_idx: usize,
        collected_args: CollectedArgs,
        pending_creds: HashMap<String, String>,
    ) {
        let script = &self.registry[script_idx];

        // First: ensure global credentials if the script requests them
        if script.manifest.global_auth {
            for key in credentials::GLOBAL_KEYS {
                let key_str = key.to_string();
                if !pending_creds.contains_key(&key_str) && credentials::get(key).is_none() {
                    self.mode = Mode::CollectingCred {
                        script_idx,
                        key: key_str,
                        resume_arg_idx: arg_idx,
                        collected_args,
                        pending_creds,
                    };
                    return;
                }
            }
        }

        // Then: ensure all required script-specific credentials are present
        for key in &script.manifest.requires_auth {
            if !pending_creds.contains_key(key) && credentials::get(key).is_none() {
                // Need to collect this credential
                self.mode = Mode::CollectingCred {
                    script_idx,
                    key: key.clone(),
                    resume_arg_idx: arg_idx,
                    collected_args,
                    pending_creds,
                };
                return;
            }
        }

        // Then: collect args one by one
        let args = &script.manifest.args;
        if arg_idx < args.len() {
            let arg = &args[arg_idx];
            // Initialize select cursor: start at default option if set
            let select_cursor = arg.default.as_deref()
                .and_then(|def| arg.options.iter().position(|o| o == def))
                .unwrap_or(0);
            // Initialize multiselect pre-selection from default (comma-separated)
            let multiselect_selected: Vec<String> = arg.default.as_deref()
                .map(|def| def.split(',').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default();
            self.mode = Mode::CollectingArgs {
                script_idx,
                arg_idx,
                collected: collected_args,
                pending_creds,
                select_cursor,
                multiselect_selected,
                validation_error: None,
            };
            return;
        }

        // All creds and args collected — execute
        self.execute_script(script_idx, collected_args, pending_creds);
    }

    pub fn execute_script(
        &mut self,
        script_idx: usize,
        collected_args: CollectedArgs,
        pending_creds: HashMap<String, String>,
    ) {
        self.mode = Mode::Running;
        let entry = &self.registry[script_idx];
        let mut lines: Vec<StyledLine> = Vec::new();
        let start = Instant::now();

        let status = executor::run(entry, &collected_args, &pending_creds, |line| {
            match line {
                OutputLine::Stdout(s) => lines.push(StyledLine { text: s, is_error: false }),
                OutputLine::Stderr(s) => lines.push(StyledLine { text: s, is_error: true }),
            }
        });

        let elapsed_ms = start.elapsed().as_millis() as u64;
        self.output_lines = lines;

        match status {
            ExitStatus::Success => {
                self.mode = Mode::ExecutionResult { exit_code: 0, elapsed_ms };
            }
            ExitStatus::Failure(code) => {
                self.mode = Mode::ExecutionResult { exit_code: code, elapsed_ms };
            }
            ExitStatus::AuthError => {
                // Delete all stored credentials for this script
                credentials::delete_all(&self.registry[script_idx].manifest.requires_auth);
                // Also delete global credentials if this script uses them
                if self.registry[script_idx].manifest.global_auth {
                    credentials::delete_all(
                        &credentials::GLOBAL_KEYS.map(str::to_string),
                    );
                }
                self.mode = Mode::AuthErrorPrompt { script_idx };
            }
        }
    }

    pub fn return_to_menu(&mut self) {
        self.mode = Mode::Menu;
        self.output_lines.clear();
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::manifest::{ArgSpec, ArgType, ScriptManifest};

    fn make_entry(name: &str, group: &str, description: &str, requires_auth: Vec<String>) -> ScriptEntry {
        ScriptEntry {
            path: PathBuf::from(format!("/tmp/{}.py", name)),
            manifest: ScriptManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: description.to_string(),
                group: group.to_string(),
                args: vec![],
                requires_auth,
                runtime: None,
                global_auth: false,
            },
        }
    }

    #[test]
    fn navigation_clamps_at_top() {
        let mut app = App::new(
            vec![make_entry("a", "g", "desc a", vec![]), make_entry("b", "g", "desc b", vec![])],
            vec![],
        );
        app.move_up(); // already at 0, should not go negative
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn navigation_clamps_at_bottom() {
        let mut app = App::new(
            vec![make_entry("a", "g", "desc a", vec![]), make_entry("b", "g", "desc b", vec![])],
            vec![],
        );
        app.move_down();
        assert_eq!(app.cursor, 1);
        app.move_down(); // already at last, should not go past
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn search_filters_visible_scripts_by_name() {
        let mut app = App::new(
            vec![
                make_entry("alpha", "group1", "first script", vec![]),
                make_entry("beta", "group2", "second script", vec![]),
            ],
            vec![],
        );
        app.update_search("alpha");
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.visible[0], 0);
    }

    #[test]
    fn search_filters_by_description() {
        let mut app = App::new(
            vec![
                make_entry("a", "g", "deploys to production", vec![]),
                make_entry("b", "g", "runs unit tests", vec![]),
            ],
            vec![],
        );
        app.update_search("tests");
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.visible[0], 1);
    }

    #[test]
    fn clear_search_restores_all_visible() {
        let mut app = App::new(
            vec![
                make_entry("alpha", "g", "first", vec![]),
                make_entry("beta", "g", "second", vec![]),
            ],
            vec![],
        );
        app.update_search("alpha");
        app.clear_search();
        assert_eq!(app.visible.len(), 2);
    }

    #[test]
    fn cursor_clamped_after_filter_reduces_visible() {
        let mut app = App::new(
            vec![
                make_entry("alpha", "g", "first", vec![]),
                make_entry("beta", "g", "second", vec![]),
            ],
            vec![],
        );
        app.cursor = 1; // point at "beta"
        app.update_search("alpha"); // only "alpha" visible
        assert_eq!(app.cursor, 0, "cursor should be clamped to last visible index");
    }

    #[test]
    fn return_to_menu_clears_output_lines() {
        let mut app = App::new(vec![], vec![]);
        app.output_lines.push(StyledLine { text: "hello".to_string(), is_error: false });
        app.mode = Mode::ExecutionResult { exit_code: 0, elapsed_ms: 10 };
        app.return_to_menu();
        assert_eq!(app.mode, Mode::Menu);
        assert!(app.output_lines.is_empty());
    }

    #[test]
    fn begin_execute_with_no_scripts_does_not_panic() {
        let mut app = App::new(vec![], vec![]);
        app.begin_execute(); // visible is empty, should no-op
        assert_eq!(app.mode, Mode::Menu);
    }

    #[test]
    fn begin_execute_with_arg_transitions_to_collecting_args() {
        let mut entry = make_entry("my_script", "g", "does stuff", vec![]);
        entry.manifest.args = vec![ArgSpec {
            name: "target".to_string(),
            prompt: "Enter target:".to_string(),
            arg_type: ArgType::Text,
            options: vec![],
            max_length: None,
            pattern: None,
            required: true,
            default: None,
        }];
        let mut app = App::new(vec![entry], vec![]);
        app.begin_execute();
        assert!(matches!(app.mode, Mode::CollectingArgs { .. }));
    }

    #[test]
    fn delete_all_does_not_panic_with_nonexistent_keys() {
        // Keys that are not in any keychain — should gracefully no-op
        credentials::delete_all(&[
            "agrr_test_nonexistent_key_1".to_string(),
            "agrr_test_nonexistent_key_2".to_string(),
        ]);
    }

    #[test]
    fn search_filters_by_group() {
        let mut app = App::new(
            vec![
                make_entry("deploy", "infra", "deploy stuff", vec![]),
                make_entry("lint", "dev-tools", "run linter", vec![]),
            ],
            vec![],
        );
        app.update_search("infra");
        assert_eq!(app.visible.len(), 1);
        assert_eq!(app.visible[0], 0);
    }

    #[test]
    fn global_auth_transitions_to_collecting_cred_for_chave() {
        let mut entry = make_entry("my_script", "g", "needs global", vec![]);
        entry.manifest.global_auth = true;
        let mut app = App::new(vec![entry], vec![]);
        app.begin_execute();
        match &app.mode {
            Mode::CollectingCred { key, .. } => {
                assert_eq!(key, "CHAVE");
            }
            other => panic!("expected CollectingCred for CHAVE, got {:?}", other),
        }
    }

    #[test]
    fn begin_execute_with_multiselect_default_preselects() {
        let mut entry = make_entry("my_script", "g", "does stuff", vec![]);
        entry.manifest.args = vec![ArgSpec {
            name: "tags".to_string(),
            prompt: "Tags?".to_string(),
            arg_type: ArgType::MultiSelect,
            options: vec!["alpha".into(), "beta".into(), "gamma".into()],
            max_length: None,
            pattern: None,
            required: false,
            default: Some("alpha,gamma".into()),
        }];
        let mut app = App::new(vec![entry], vec![]);
        app.begin_execute();
        match &app.mode {
            Mode::CollectingArgs { multiselect_selected, .. } => {
                assert_eq!(multiselect_selected, &vec!["alpha".to_string(), "gamma".to_string()]);
            }
            other => panic!("expected CollectingArgs, got {:?}", other),
        }
    }
}
