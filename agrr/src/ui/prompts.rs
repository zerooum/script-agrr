use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use super::theme::{TN_FG, TN_MUTED, TN_BLUE, TN_YELLOW, TN_ORANGE, TN_RED, TN_GREEN, is_masked_field};
use super::layout::centered_rect;

// ─── Arg prompt ───────────────────────────────────────────────────────────────

pub(super) fn render_arg_prompt(frame: &mut Frame, app: &App, arg_idx: usize) {
    let Mode::CollectingArgs {
        script_idx,
        collected,
        ..
    } = &app.mode
    else {
        return;
    };

    let script = &app.registry[*script_idx];
    let arg = &script.manifest.args[arg_idx];
    let current_input = collected.get(&arg.name).map(String::as_str).unwrap_or("");
    let total = script.manifest.args.len();

    let area = centered_rect(65, 50, frame.area());
    let block = Block::default()
        .title(Span::styled(
            format!("  {}  ", script.manifest.name),
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_BLUE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!("  argumento {} de {}", arg_idx + 1, total),
            Style::default().fg(TN_MUTED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                &arg.prompt,
                Style::default().fg(TN_FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if arg.options.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  > ", Style::default().fg(TN_ORANGE)),
            Span::styled(
                format!("{}_", current_input),
                Style::default().fg(TN_YELLOW),
            ),
        ]));
    } else {
        for opt in &arg.options {
            if opt.as_str() == current_input {
                lines.push(Line::from(vec![
                    Span::styled("  > ", Style::default().fg(TN_BLUE)),
                    Span::styled(
                        opt.clone(),
                        Style::default()
                            .fg(TN_BLUE)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(opt.clone(), Style::default().fg(TN_FG)),
                ]));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  > ", Style::default().fg(TN_ORANGE)),
            Span::styled(
                format!("{}_", current_input),
                Style::default().fg(TN_YELLOW),
            ),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Credential prompt ────────────────────────────────────────────────────────

pub(super) fn render_cred_prompt(frame: &mut Frame, app: &App, cred_key: &str) {
    let Mode::CollectingCred { script_idx, pending_creds, .. } = &app.mode else {
        return;
    };
    let script = &app.registry[*script_idx];
    let input_val = pending_creds
        .get(&format!("__input__{}", cred_key))
        .map(String::as_str)
        .unwrap_or("");
    let masked = is_masked_field(cred_key);
    let display = if masked {
        "*".repeat(input_val.len())
    } else {
        input_val.to_string()
    };

    let area = centered_rect(65, 50, frame.area());
    let block = Block::default()
        .title(Span::styled(
            format!("  {}  ", script.manifest.name),
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_BLUE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tipo_hint = if masked {
        "entrada mascarada"
    } else {
        "entrada visivel"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  script  ", Style::default().fg(TN_MUTED)),
            Span::styled(
                &script.manifest.name,
                Style::default().fg(TN_FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  chave   ", Style::default().fg(TN_MUTED)),
            Span::styled(
                cred_key.to_string(),
                Style::default()
                    .fg(TN_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(TN_ORANGE)),
            Span::styled(
                format!("{}_", display),
                Style::default().fg(TN_YELLOW),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} — pressione Enter para confirmar", tipo_hint),
            Style::default()
                .fg(TN_MUTED)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Ask save credential ──────────────────────────────────────────────────────

pub(super) fn render_ask_save(frame: &mut Frame, _app: &App, save_key: &str) {
    let area = centered_rect(65, 50, frame.area());
    let block = Block::default()
        .title(Span::styled(
            "  salvar credencial?  ",
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_BLUE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Salvar ", Style::default().fg(TN_FG)),
            Span::styled(
                save_key.to_string(),
                Style::default()
                    .fg(TN_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " no keychain para proximas execucoes?",
                Style::default().fg(TN_FG),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[s]",
                Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  sim, salvar    ", Style::default().fg(TN_FG)),
            Span::styled(
                "[N]",
                Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  nao, usar so agora", Style::default().fg(TN_MUTED)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}
