use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode, StyledLine};
use crate::credentials;
use crate::manifest::Language;

// ─── Masking helper ───────────────────────────────────────────────────────────

/// Returns true if the credential field should be masked in the TUI.
/// Applies to keys containing SENHA, PASSWORD, or SECRET (case-insensitive).
/// Login-style keys (CHAVE, USUARIO, LOGIN, API_KEY, TOKEN, etc.) are NOT masked.
fn is_masked_field(key: &str) -> bool {
    let u = key.to_uppercase();
    u.contains("SENHA") || u.contains("PASSWORD") || u.contains("SECRET")
}

// ─── Tokyo Night palette ──────────────────────────────────────────────────────

const TN_FG: Color     = Color::Rgb(192, 202, 245);
const TN_MUTED: Color  = Color::Rgb(86, 95, 137);
const TN_BLUE: Color   = Color::Rgb(122, 162, 247);
const TN_PURPLE: Color = Color::Rgb(187, 154, 247);
const TN_CYAN: Color   = Color::Rgb(125, 207, 255);
const TN_GREEN: Color  = Color::Rgb(158, 206, 106);
const TN_RED: Color    = Color::Rgb(247, 118, 142);
const TN_YELLOW: Color = Color::Rgb(224, 175, 104);
const TN_ORANGE: Color = Color::Rgb(255, 158, 100);
const TN_SEL: Color    = Color::Rgb(54, 74, 130);

// ─── Span helpers ─────────────────────────────────────────────────────────────

fn key(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD))
}

fn desc(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(TN_MUTED))
}

// ─── Top-level dispatcher ─────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    if !app.warnings_dismissed && !app.warnings.is_empty() {
        render_warnings(frame, app, frame.area());
        return;
    }

    match &app.mode {
        Mode::Menu | Mode::Search => render_menu(frame, app, frame.area()),
        Mode::CollectingArgs { arg_idx, .. } => render_arg_prompt(frame, app, *arg_idx),
        Mode::CollectingCred { key, .. } => render_cred_prompt(frame, app, key),
        Mode::AskSaveCred { key, .. } => render_ask_save(frame, app, key),
        Mode::Running => render_output(frame, app, None, 0),
        Mode::ExecutionResult { exit_code, elapsed_ms } => {
            render_output(frame, app, Some(*exit_code), *elapsed_ms)
        }
        Mode::AuthErrorPrompt { .. } => render_auth_error(frame, app),
        Mode::CredManager { cursor } => render_cred_manager(frame, app, *cursor),
        Mode::CredManagerSaving { script_idx, key, .. } => {
            render_cred_manager_saving(frame, app, *script_idx, key)
        }
        Mode::CredManagerClearConfirm { script_idx, .. } => {
            render_cred_manager_clear_confirm(frame, app, *script_idx)
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

// ─── Main menu ────────────────────────────────────────────────────────────────

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    // Outer split: main content + 1-line borderless footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    // Inner split: 35% script list | 65% detail panel
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    if matches!(app.mode, Mode::Search) {
        // Left column: list + search input
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(cols[0]);

        render_script_list(frame, app, left[0]);
        render_search_input(frame, app, left[1]);
        render_footer(frame, outer[1], true);
    } else {
        render_script_list(frame, app, cols[0]);
        render_footer(frame, outer[1], false);
    }

    render_detail_panel(frame, app, cols[1]);
}

// ─── Script list (left panel) ─────────────────────────────────────────────────

fn render_script_list(frame: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut list_state = ListState::default();
    let mut render_idx: usize = 0;

    let mut current_group = "";
    for (logical_pos, &script_idx) in app.visible.iter().enumerate() {
        let script = &app.registry[script_idx];
        let group = script.manifest.group.as_str();

        if group != current_group {
            items.push(ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    format!("◆ {}", group.to_uppercase()),
                    Style::default().fg(TN_PURPLE).add_modifier(Modifier::BOLD),
                ),
            ])));
            current_group = group;
            render_idx += 1;
        }

        if logical_pos == app.cursor {
            list_state.select(Some(render_idx));
        }

        let name_style = if logical_pos == app.cursor {
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::styled(&script.manifest.name, name_style),
        ])));
        render_idx += 1;
    }

    if app.visible.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  nenhum resultado",
            Style::default()
                .fg(TN_MUTED)
                .add_modifier(Modifier::ITALIC),
        ))));
    }

    let is_searching = matches!(app.mode, Mode::Search);
    let border_color = if is_searching { TN_YELLOW } else { TN_BLUE };

    let block = Block::default()
        .title(Span::styled(
            " agrr ",
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(TN_SEL)
                .fg(TN_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

// ─── Detail panel (right panel) ───────────────────────────────────────────────

fn render_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " detalhes ",
            Style::default().fg(TN_MUTED),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_MUTED));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.visible.is_empty() {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);
        frame.render_widget(
            Paragraph::new(Span::styled(
                "nenhum script disponível",
                Style::default()
                    .fg(TN_MUTED)
                    .add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Center),
            vert[1],
        );
        return;
    }

    let script = &app.registry[app.visible[app.cursor]];
    let m = &script.manifest;

    let mut lines: Vec<Line> = Vec::new();

    // Name + version
    lines.push(Line::from(vec![
        Span::styled(
            &m.name,
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("v{}", m.version),
            Style::default().fg(TN_MUTED),
        ),
    ]));

    // Description
    lines.push(Line::from(Span::styled(
        &m.description,
        Style::default().fg(TN_FG),
    )));

    lines.push(Line::from(""));

    // Group + runtime in side-by-side label rows
    lines.push(Line::from(vec![
        Span::styled("grupo    ", Style::default().fg(TN_MUTED)),
        Span::styled(&m.group, Style::default().fg(TN_CYAN)),
    ]));

    let runtime_text = match &m.runtime {
        None => "nativo (binário compilado)".to_string(),
        Some(rt) => {
            let lang = match rt.language {
                Language::Python => "Python",
                Language::Node => "Node.js",
            };
            format!("{} >= {}", lang, rt.min_version)
        }
    };
    lines.push(Line::from(vec![
        Span::styled("runtime  ", Style::default().fg(TN_MUTED)),
        Span::styled(runtime_text, Style::default().fg(TN_CYAN)),
    ]));

    // Credentials
    if !m.requires_auth.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "credenciais",
            Style::default()
                .fg(TN_MUTED)
                .add_modifier(Modifier::BOLD),
        )));
        for cred_key in &m.requires_auth {
            lines.push(Line::from(vec![
                Span::styled("  * ", Style::default().fg(TN_YELLOW)),
                Span::styled(cred_key.clone(), Style::default().fg(TN_FG)),
            ]));
        }
    }

    // Arguments
    if !m.args.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "argumentos",
            Style::default()
                .fg(TN_MUTED)
                .add_modifier(Modifier::BOLD),
        )));
        for arg in &m.args {
            let suffix = if arg.options.is_empty() {
                format!("  {}", arg.prompt)
            } else {
                format!("  [{}]", arg.options.join(" | "))
            };
            lines.push(Line::from(vec![
                Span::styled("  > ", Style::default().fg(TN_ORANGE)),
                Span::styled(
                    arg.name.clone(),
                    Style::default()
                        .fg(TN_FG)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(suffix, Style::default().fg(TN_MUTED)),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Footer (borderless, context-sensitive) ───────────────────────────────────

fn render_footer(frame: &mut Frame, area: Rect, searching: bool) {
    let line = if searching {
        Line::from(vec![
            desc("  "),
            key("Esc"),
            desc(" cancelar  "),
            key("↑↓"),
            desc(" navegar  "),
            key("Enter"),
            desc(" executar"),
        ])
    } else {
        Line::from(vec![
            desc("  "),
            key("↑↓"),
            desc(" navegar  "),
            key("Enter"),
            desc(" executar  "),
            key("/"),
            desc(" buscar  "),
            key("c"),
            desc(" credenciais  "),
            key("q"),
            desc(" sair"),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}

// ─── Search input ─────────────────────────────────────────────────────────────

fn render_search_input(frame: &mut Frame, app: &App, area: Rect) {
    let query_display = format!(" {}_", app.search_query);
    let block = Block::default()
        .title(Span::styled(
            " / buscar ",
            Style::default()
                .fg(TN_YELLOW)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_YELLOW));
    let para = Paragraph::new(query_display)
        .block(block)
        .style(Style::default().fg(TN_YELLOW));
    frame.render_widget(para, area);
}

// ─── Prompts ──────────────────────────────────────────────────────────────────

fn render_arg_prompt(frame: &mut Frame, app: &App, arg_idx: usize) {
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

fn render_cred_prompt(frame: &mut Frame, app: &App, cred_key: &str) {
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

fn render_ask_save(frame: &mut Frame, _app: &App, save_key: &str) {
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

// ─── Output pane ──────────────────────────────────────────────────────────────

fn render_output(frame: &mut Frame, app: &App, exit_code: Option<i32>, elapsed_ms: u64) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .output_lines
        .iter()
        .map(|l: &StyledLine| {
            let style = if l.is_error {
                Style::default().fg(TN_RED)
            } else {
                Style::default().fg(TN_FG)
            };
            ListItem::new(Line::from(Span::styled(&l.text, style)))
        })
        .collect();

    let total = items.len();

    let title = match exit_code {
        None => " \u{28ff} executando\u{2026} ".to_string(),
        Some(0) => format!(
            " \u{2713} concluido  ({:.1}s) ",
            elapsed_ms as f64 / 1000.0
        ),
        Some(c) => format!(
            " \u{2717} codigo {}  ({:.1}s) ",
            c,
            elapsed_ms as f64 / 1000.0
        ),
    };

    let border_color = match exit_code {
        None => TN_BLUE,
        Some(0) => TN_GREEN,
        Some(_) => TN_RED,
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let visible_height = chunks[0].height.saturating_sub(2) as usize;
    let scroll_offset = if total > 0 {
        app.output_scroll.min(total - 1)
    } else {
        0
    };

    let mut list_state = ListState::default();
    *list_state.offset_mut() = scroll_offset;

    frame.render_stateful_widget(
        List::new(items).block(block),
        chunks[0],
        &mut list_state,
    );

    // Context-sensitive footer
    let remaining = total.saturating_sub(scroll_offset + visible_height);
    let footer_line = if exit_code.is_some() {
        let mut spans = vec![desc("  "), key("\u{2191}\u{2193}"), desc(" rolar  "), key("Esc"), desc(" voltar ao menu")];
        if remaining > 0 {
            spans.push(Span::styled(
                format!("     \u{2193} {} linhas abaixo", remaining),
                Style::default().fg(TN_ORANGE),
            ));
        }
        Line::from(spans)
    } else {
        Line::from(desc("  aguardando script\u{2026}"))
    };

    frame.render_widget(Paragraph::new(footer_line), chunks[1]);
}

// ─── Auth error ───────────────────────────────────────────────────────────────

fn render_auth_error(frame: &mut Frame, _app: &App) {
    let area = centered_rect(65, 50, frame.area());
    let block = Block::default()
        .title(Span::styled(
            "  erro de autenticacao  ",
            Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_RED));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Credenciais invalidas.",
            Style::default().fg(TN_FG),
        )),
        Line::from(Span::styled(
            "  As credenciais salvas foram removidas.",
            Style::default().fg(TN_MUTED),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Deseja tentar novamente com novas credenciais?",
            Style::default().fg(TN_FG),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[S]",
                Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  sim, tentar novamente    ",
                Style::default().fg(TN_FG),
            ),
            Span::styled(
                "[n]",
                Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  nao, voltar ao menu", Style::default().fg(TN_MUTED)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ─── Credential manager ───────────────────────────────────────────────────────

fn render_cred_manager(frame: &mut Frame, app: &App, cursor: usize) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    // ── Left: list (cursor 0 = Globais, cursor 1+ = scripts) ────────────────
    let scripts_with_auth: Vec<usize> = (0..app.registry.len())
        .filter(|&i| !app.registry[i].manifest.requires_auth.is_empty())
        .collect();

    // Global entry at render index 0
    let global_chave_ok = credentials::get("CHAVE").is_some();
    let global_senha_ok = credentials::get("SENHA").is_some();
    let global_all = global_chave_ok && global_senha_ok;
    let global_status = if global_all {
        Span::styled(" ✓", Style::default().fg(TN_GREEN))
    } else {
        Span::styled(" ✗", Style::default().fg(TN_RED))
    };
    let global_name_style = if cursor == 0 {
        Style::default().fg(TN_PURPLE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)
    };
    let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(vec![
        Span::raw("   "),
        Span::styled("◆ Globais (agrr)", global_name_style),
        global_status,
    ]))];

    for (pos, &script_idx) in scripts_with_auth.iter().enumerate() {
        let m = &app.registry[script_idx].manifest;
        let all_saved = m.requires_auth.iter().all(|k| credentials::get(k).is_some());
        let status_span = if all_saved {
            Span::styled(" ✓", Style::default().fg(TN_GREEN))
        } else {
            Span::styled(" ✗", Style::default().fg(TN_RED))
        };
        let name_style = if cursor == pos + 1 {
            Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::styled(&m.name, name_style),
            status_span,
        ])));
    }

    let mut list_state = ListState::default();
    list_state.select(Some(cursor));

    let block = Block::default()
        .title(Span::styled(
            " credenciais ",
            Style::default().fg(TN_PURPLE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_PURPLE));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(TN_SEL).fg(TN_PURPLE).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, cols[0], &mut list_state);

    // ── Right: detail panel ───────────────────────────────────────────────────────────
    let detail_block = Block::default()
        .title(Span::styled(" detalhes ", Style::default().fg(TN_MUTED)))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_MUTED));

    let inner = detail_block.inner(cols[1]);
    frame.render_widget(detail_block, cols[1]);

    let lines: Vec<Line> = if cursor == 0 {
        cred_manager_global_detail(global_chave_ok, global_senha_ok)
    } else if let Some(&script_idx) = scripts_with_auth.get(cursor - 1) {
        cred_manager_script_detail(&app.registry[script_idx].manifest)
    } else {
        vec![Line::from(Span::styled(
            "nenhum script com credenciais",
            Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
        ))]
    };

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    // ── Footer ────────────────────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            desc("  "),
            key("↑↓"),
            desc(" navegar  "),
            key("Enter"),
            desc(" salvar  "),
            key("l"),
            desc(" limpar  "),
            key("Esc"),
            desc(" voltar"),
        ])),
        outer[1],
    );
}

fn cred_manager_global_detail(chave_ok: bool, senha_ok: bool) -> Vec<Line<'static>> {
    let all_saved = chave_ok && senha_ok;
    let any_saved = chave_ok || senha_ok;
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "Globais (agrr)",
            Style::default().fg(TN_PURPLE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Credenciais compartilhadas entre todos os scripts com global_auth.",
            Style::default().fg(TN_FG),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "chaves de credencial",
            Style::default().fg(TN_MUTED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for (cred_key, saved) in &[("CHAVE", chave_ok), ("SENHA", senha_ok)] {
        let (icon, status_text, color) = if *saved {
            ("  ✓ ", "salvo", TN_GREEN)
        } else {
            ("  ✗ ", "não salvo", TN_RED)
        };
        lines.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::styled(cred_key.to_string(), Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(status_text, Style::default().fg(color)),
        ]));
    }
    lines.push(Line::from(""));
    if all_saved {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[l]", Style::default().fg(TN_RED).add_modifier(Modifier::BOLD)),
            Span::styled("  limpar credenciais globais", Style::default().fg(TN_FG)),
        ]));
    } else if any_saved {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  salvar pendentes    ", Style::default().fg(TN_FG)),
            Span::styled("[l]", Style::default().fg(TN_RED).add_modifier(Modifier::BOLD)),
            Span::styled("  limpar as salvas", Style::default().fg(TN_MUTED)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  salvar CHAVE e SENHA globais", Style::default().fg(TN_FG)),
        ]));
    }
    lines
}

fn cred_manager_script_detail(m: &crate::manifest::ScriptManifest) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(m.name.clone(), Style::default().fg(TN_BLUE).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(format!("v{}", m.version), Style::default().fg(TN_MUTED)),
        ]),
        Line::from(Span::styled(m.description.clone(), Style::default().fg(TN_FG))),
        Line::from(""),
        Line::from(Span::styled(
            "chaves de credencial",
            Style::default().fg(TN_MUTED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for cred_key in &m.requires_auth {
        let saved = credentials::get(cred_key).is_some();
        let (icon, status_text, color) = if saved {
            ("  ✓ ", "salvo", TN_GREEN)
        } else {
            ("  ✗ ", "não salvo", TN_RED)
        };
        lines.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::styled(cred_key.clone(), Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(status_text, Style::default().fg(color)),
        ]));
    }
    lines.push(Line::from(""));
    let all_saved = m.requires_auth.iter().all(|k| credentials::get(k).is_some());
    let any_saved = m.requires_auth.iter().any(|k| credentials::get(k).is_some());
    if all_saved {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[l]", Style::default().fg(TN_RED).add_modifier(Modifier::BOLD)),
            Span::styled("  limpar todas as credenciais", Style::default().fg(TN_FG)),
        ]));
    } else if any_saved {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  salvar pendentes    ", Style::default().fg(TN_FG)),
            Span::styled("[l]", Style::default().fg(TN_RED).add_modifier(Modifier::BOLD)),
            Span::styled("  limpar as salvas", Style::default().fg(TN_MUTED)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  salvar credenciais", Style::default().fg(TN_FG)),
        ]));
    }
    lines
}

fn render_cred_manager_saving(
    frame: &mut Frame,
    app: &App,
    script_idx: Option<usize>,
    cred_key: &str,
) {
    let Mode::CredManagerSaving { input, .. } = &app.mode else { return; };
    let masked = is_masked_field(cred_key);
    let display = if masked { "*".repeat(input.len()) } else { input.clone() };
    let tipo_hint = if masked { "entrada mascarada" } else { "entrada visível" };

    let (script_name, total, key_pos, already_saved) = match script_idx {
        None => {
            let keys = credentials::GLOBAL_KEYS;
            let pos = keys.iter().position(|&k| k == cred_key).unwrap_or(0);
            let saved = keys.iter().filter(|&&k| credentials::get(k).is_some()).count();
            ("Globais (agrr)".to_string(), keys.len(), pos, saved)
        }
        Some(idx) => {
            let m = &app.registry[idx].manifest;
            let pos = m.requires_auth.iter().position(|k| k == cred_key).unwrap_or(0);
            let saved = m.requires_auth.iter().filter(|k| credentials::get(*k).is_some()).count();
            (m.name.clone(), m.requires_auth.len(), pos, saved)
        }
    };

    let area = centered_rect(65, 55, frame.area());
    let block = Block::default()
        .title(Span::styled(
            "  gerenciar credenciais  ",
            Style::default().fg(TN_PURPLE).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_PURPLE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  origem  ", Style::default().fg(TN_MUTED)),
            Span::styled(script_name, Style::default().fg(TN_FG).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  chave   ", Style::default().fg(TN_MUTED)),
            Span::styled(cred_key.to_string(), Style::default().fg(TN_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  ({} de {}, {} já salvas)", key_pos + 1, total, already_saved),
                Style::default().fg(TN_MUTED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(TN_ORANGE)),
            Span::styled(format!("{}_", display), Style::default().fg(TN_YELLOW)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} — pressione Enter para salvar", tipo_hint),
            Style::default().fg(TN_MUTED).add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(TN_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("  salvar    ", Style::default().fg(TN_FG)),
            Span::styled("[Esc]", Style::default().fg(TN_MUTED).add_modifier(Modifier::BOLD)),
            Span::styled("  cancelar", Style::default().fg(TN_MUTED)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_cred_manager_clear_confirm(frame: &mut Frame, app: &App, script_idx: Option<usize>) {
    let (script_name, keys_to_clear): (String, Vec<String>) = match script_idx {
        None => (
            "Globais (agrr)".to_string(),
            credentials::GLOBAL_KEYS
                .iter()
                .filter(|&&k| credentials::get(k).is_some())
                .map(|k| k.to_string())
                .collect(),
        ),
        Some(idx) => {
            let m = &app.registry[idx].manifest;
            let keys = m.requires_auth.iter()
                .filter(|k| credentials::get(*k).is_some())
                .cloned()
                .collect();
            (m.name.clone(), keys)
        }
    };

    let area = centered_rect(65, 50, frame.area());
    let block = Block::default()
        .title(Span::styled(
            "  limpar credenciais?  ",
            Style::default().fg(TN_RED).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TN_RED));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Limpar todas as credenciais de ", Style::default().fg(TN_FG)),
            Span::styled(script_name, Style::default().fg(TN_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("?", Style::default().fg(TN_FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Chaves que serão removidas:", Style::default().fg(TN_MUTED))),
    ];

    for cred_key in &keys_to_clear {
        lines.push(Line::from(vec![
            Span::styled("    • ", Style::default().fg(TN_RED)),
            Span::styled(cred_key.clone(), Style::default().fg(TN_FG)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("[s]", Style::default().fg(TN_RED).add_modifier(Modifier::BOLD)),
        Span::styled("  sim, limpar tudo    ", Style::default().fg(TN_FG)),
        Span::styled("[N]", Style::default().fg(TN_MUTED).add_modifier(Modifier::BOLD)),
        Span::styled("  cancelar", Style::default().fg(TN_MUTED)),
    ]));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

// ─── Layout helpers ────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
