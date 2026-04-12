use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use crate::manifest::Language;
use super::theme::{
    TN_FG, TN_MUTED, TN_BLUE, TN_PURPLE, TN_CYAN, TN_YELLOW, TN_ORANGE, TN_SEL,
    key, desc,
};

// ─── Main menu ────────────────────────────────────────────────────────────────

pub(super) fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
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
