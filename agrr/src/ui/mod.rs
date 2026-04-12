mod theme;
mod layout;
mod menu;
mod prompts;
mod output;
mod cred_mgr;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, Mode};
use theme::{TN_FG, TN_YELLOW};

// ─── Top-level dispatcher ─────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    if !app.warnings_dismissed && !app.warnings.is_empty() {
        render_warnings(frame, app, frame.area());
        return;
    }

    match &app.mode {
        Mode::Menu | Mode::Search => menu::render_menu(frame, app, frame.area()),
        Mode::CollectingArgs { arg_idx, .. } => prompts::render_arg_prompt(frame, app, *arg_idx),
        Mode::CollectingCred { key, .. } => prompts::render_cred_prompt(frame, app, key),
        Mode::AskSaveCred { key, .. } => prompts::render_ask_save(frame, app, key),
        Mode::Running => output::render_output(frame, app, None, 0),
        Mode::ExecutionResult { exit_code, elapsed_ms } => {
            output::render_output(frame, app, Some(*exit_code), *elapsed_ms)
        }
        Mode::AuthErrorPrompt { .. } => output::render_auth_error(frame, app),
        Mode::CredManager { cursor } => cred_mgr::render_cred_manager(frame, app, *cursor),
        Mode::CredManagerSaving { script_idx, key, .. } => {
            cred_mgr::render_cred_manager_saving(frame, app, *script_idx, key)
        }
        Mode::CredManagerClearConfirm { script_idx, .. } => {
            cred_mgr::render_cred_manager_clear_confirm(frame, app, *script_idx)
        }
        Mode::Quit => {}
    }
}

// ─── Warnings panel ───────────────────────────────────────────────────────────

fn render_warnings(frame: &mut Frame, app: &App, area: Rect) {
    let count = app.warnings.len();
    let items: Vec<ListItem> = app
        .warnings
        .iter()
        .map(|w| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    "  ⚠  ",
                    Style::default().fg(TN_YELLOW).add_modifier(Modifier::BOLD),
                ),
                Span::styled(w.to_string(), Style::default().fg(TN_FG)),
            ]))
        })
        .collect();

    let title = format!(
        " ⚠  {} aviso{}  —  pressione qualquer tecla para continuar ",
        count,
        if count == 1 { "" } else { "s" }
    );
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_YELLOW));

    frame.render_widget(List::new(items).block(block), area);
}
