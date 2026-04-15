use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use crate::manifest::ArgType;
use super::theme::{TN_FG, TN_MUTED, TN_BLUE, TN_YELLOW, TN_ORANGE, TN_RED, TN_GREEN, is_masked_field};
use super::layout::centered_rect;
use crate::credentials::global_cred_constraint;

// ─── Arg prompt ───────────────────────────────────────────────────────────────

pub(super) fn render_arg_prompt(frame: &mut Frame, app: &App, arg_idx: usize, selected_subcommand: Option<&str>) {
    let Mode::CollectingArgs {
        script_idx,
        collected,
        select_cursor,
        multiselect_selected,
        validation_error,
        ..
    } = &app.mode
    else {
        return;
    };

    let script = &app.registry[*script_idx];
    let effective = script.manifest.effective_args(selected_subcommand);
    let arg = &effective[arg_idx];
    let total = effective.len();

    let area = centered_rect(65, 55, frame.area());
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

    match &arg.arg_type {
        ArgType::Text => {
            let current_input = collected.get(&arg.name).map(String::as_str).unwrap_or("");

            // Hint line for default / constraints
            let mut hint_parts: Vec<Span> = vec![Span::raw("  ")];
            if let Some(def) = &arg.default {
                hint_parts.push(Span::styled(
                    format!("(padrão: {})  ", def),
                    Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
                ));
            }
            if let Some(max) = arg.max_length {
                hint_parts.push(Span::styled(
                    format!("max {} chars  ", max),
                    Style::default().fg(TN_MUTED),
                ));
            }
            if hint_parts.len() > 1 {
                lines.push(Line::from(hint_parts));
                lines.push(Line::from(""));
            }

            lines.push(Line::from(vec![
                Span::styled("  > ", Style::default().fg(TN_ORANGE)),
                Span::styled(
                    format!("{}_", current_input),
                    Style::default().fg(TN_YELLOW),
                ),
            ]));

            if let Some(err) = validation_error {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        err.clone(),
                        Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }

        ArgType::Select => {
            for (i, opt) in arg.options.iter().enumerate() {
                if i == *select_cursor {
                    lines.push(Line::from(vec![
                        Span::styled("  > ", Style::default().fg(TN_BLUE)),
                        Span::styled(
                            opt.clone(),
                            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
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
            lines.push(Line::from(Span::styled(
                "  ↑↓ navegar  Enter confirmar",
                Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
            )));
        }

        ArgType::MultiSelect => {
            for (i, opt) in arg.options.iter().enumerate() {
                let is_selected = multiselect_selected.iter().any(|s| s == opt);
                let checkbox = if is_selected { "☑" } else { "☐" };
                if i == *select_cursor {
                    lines.push(Line::from(vec![
                        Span::styled("  > ", Style::default().fg(TN_BLUE)),
                        Span::styled(
                            format!("{} {}", checkbox, opt),
                            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!("{} {}", checkbox, opt),
                            if is_selected {
                                Style::default().fg(TN_GREEN)
                            } else {
                                Style::default().fg(TN_FG)
                            },
                        ),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ↑↓ navegar  Espaço selecionar  Enter confirmar",
                Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
            )));
            if let Some(err) = validation_error {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        err.clone(),
                        Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Subcommand selection ─────────────────────────────────────────────────────

pub(super) fn render_subcommand_selection(frame: &mut Frame, app: &App, cursor: usize) {
    let Mode::SelectingSubcommand { script_idx, .. } = &app.mode else { return; };
    let script = &app.registry[*script_idx];
    let subcommands = &script.manifest.subcommands;

    let area = centered_rect(65, 55, frame.area());
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
            "  selecione uma funcionalidade:",
            Style::default().fg(TN_MUTED),
        )),
        Line::from(""),
    ];

    for (i, subcmd) in subcommands.iter().enumerate() {
        if i == cursor {
            let mut spans = vec![
                Span::styled("  > ", Style::default().fg(TN_BLUE)),
                Span::styled(
                    subcmd.name.clone(),
                    Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
                ),
            ];
            if let Some(desc) = &subcmd.description {
                spans.push(Span::styled(
                    format!("  —  {}", desc),
                    Style::default().fg(TN_MUTED),
                ));
            }
            lines.push(Line::from(spans));
        } else {
            let mut spans = vec![
                Span::raw("    "),
                Span::styled(subcmd.name.clone(), Style::default().fg(TN_FG)),
            ];
            if let Some(desc) = &subcmd.description {
                spans.push(Span::styled(
                    format!("  —  {}", desc),
                    Style::default().fg(TN_MUTED),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ↑↓ navegar  Enter confirmar  Esc cancelar",
        Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
    )));

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Credential prompt ────────────────────────────────────────────────────────

pub(super) fn render_cred_prompt(frame: &mut Frame, app: &App, cred_key: &str) {
    let Mode::CollectingCred { script_idx, pending_creds, validation_error, .. } = &app.mode else {
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

    // Build constraint hint for known global credential fields
    let constraint_hint: Option<String> = global_cred_constraint(cred_key).map(|con| {
        let mut parts = vec![format!("max {} chars", con.max_length)];
        if con.pattern.is_some() {
            parts.push("apenas dígitos".to_string());
        }
        parts.join("  ")
    });

    let mut lines: Vec<Line> = vec![
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
    ];

    if let Some(hint) = constraint_hint {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(hint, Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  > ", Style::default().fg(TN_ORANGE)),
        Span::styled(
            format!("{}_", display),
            Style::default().fg(TN_YELLOW),
        ),
    ]));

    if let Some(err) = validation_error {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                err.clone(),
                Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {} — pressione Enter para confirmar", tipo_hint),
        Style::default()
            .fg(TN_MUTED)
            .add_modifier(Modifier::ITALIC),
    )));

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
