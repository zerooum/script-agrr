use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, StyledLine};
use super::theme::{TN_FG, TN_MUTED, TN_BLUE, TN_GREEN, TN_RED, TN_ORANGE, key, desc};
use super::layout::centered_rect;

// ─── Output pane ──────────────────────────────────────────────────────────────

pub(super) fn render_output(frame: &mut Frame, app: &App, exit_code: Option<i32>, elapsed_ms: u64) {
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

pub(super) fn render_auth_error(frame: &mut Frame, _app: &App) {
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
