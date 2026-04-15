mod theme;
mod layout;
mod menu;
mod prompts;
mod output;
mod cred_mgr;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
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
        Mode::SelectingSubcommand { cursor, .. } => {
            prompts::render_subcommand_selection(frame, app, *cursor)
        }
        Mode::CollectingArgs { arg_idx, selected_subcommand, .. } => {
            prompts::render_arg_prompt(frame, app, *arg_idx, selected_subcommand.as_deref())
        }
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

    // Pre-wrap text manually: first chunk gets the icon, continuation lines get
    // spaces matching the icon width so everything aligns nicely.
    let icon = "  ⚠  ";
    let icon_width = icon.chars().count();
    let inner_w = (area.width.saturating_sub(2 + icon_width as u16)) as usize;
    let indent = " ".repeat(icon_width);

    let mut lines: Vec<Line> = Vec::new();
    for w in &app.warnings {
        let text = w.to_string();
        let text = text.as_str();
        if inner_w == 0 || text.len() <= inner_w {
            lines.push(Line::from(vec![
                Span::styled(icon, Style::default().fg(TN_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(text.to_string(), Style::default().fg(TN_FG)),
            ]));
        } else {
            // Word-wrap the text into chunks
            let mut chunks: Vec<&str> = Vec::new();
            let mut start = 0;
            while start < text.len() {
                let end = (start + inner_w).min(text.len());
                // Walk back to last space to avoid splitting words
                let split = if end < text.len() {
                    text[start..end].rfind(' ').map(|i| start + i + 1).unwrap_or(end)
                } else {
                    end
                };
                chunks.push(text[start..split.min(text.len())].trim_end());
                start = split.min(text.len());
            }
            for (i, chunk) in chunks.into_iter().enumerate() {
                if i == 0 {
                    lines.push(Line::from(vec![
                        Span::styled(icon, Style::default().fg(TN_YELLOW).add_modifier(Modifier::BOLD)),
                        Span::styled(chunk.to_string(), Style::default().fg(TN_FG)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw(indent.clone()),
                        Span::styled(chunk.to_string(), Style::default().fg(TN_FG)),
                    ]));
                }
            }
        }
    }

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

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}
