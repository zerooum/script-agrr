use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use crate::manifest::ScriptManifest;
use crate::credentials;
use super::theme::{
    TN_FG, TN_MUTED, TN_BLUE, TN_PURPLE, TN_GREEN, TN_RED, TN_YELLOW, TN_ORANGE, TN_SEL,
    is_masked_field, key, desc,
};
use super::layout::centered_rect;

// ─── Credential manager ───────────────────────────────────────────────────────

pub(super) fn render_cred_manager(frame: &mut Frame, app: &App, cursor: usize) {
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

    // ── Right: detail panel ───────────────────────────────────────────────────
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

fn cred_manager_script_detail(m: &ScriptManifest) -> Vec<Line<'static>> {
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

// ─── Credential manager — saving ─────────────────────────────────────────────

pub(super) fn render_cred_manager_saving(
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

// ─── Credential manager — clear confirm ──────────────────────────────────────

pub(super) fn render_cred_manager_clear_confirm(frame: &mut Frame, app: &App, script_idx: Option<usize>) {
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
