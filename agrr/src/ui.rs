use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode, StyledLine};

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
        Mode::ExecutionResult { exit_code, elapsed_ms } => render_output(frame, app, Some(*exit_code), *elapsed_ms),
        Mode::AuthErrorPrompt { .. } => render_auth_error(frame, app),
        Mode::Quit => {}
    }
}

// ─── Warnings panel ───────────────────────────────────────────────────────────

fn render_warnings(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .warnings
        .iter()
        .map(|w| {
            ListItem::new(Line::from(vec![
                Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                Span::raw(w.to_string()),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(" Avisos de carregamento — pressione qualquer tecla para continuar ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

// ─── Main menu ────────────────────────────────────────────────────────────────

fn render_menu(frame: &mut Frame, app: &App, area: Rect) {
    if matches!(app.mode, Mode::Search) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3), Constraint::Length(1)])
            .split(area);
        render_script_list(frame, app, chunks[0]);
        render_search_input(frame, app, chunks[1]);
        render_search_hints(frame, chunks[2]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);
        render_script_list(frame, app, chunks[0]);
        render_status_bar(frame, app, chunks[1]);
    }
}

fn render_script_list(frame: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut list_state = ListState::default();
    let mut render_idx = 0;

    // Group scripts
    let mut current_group = "";
    for (logical_pos, &script_idx) in app.visible.iter().enumerate() {
        let script = &app.registry[script_idx];
        let group = script.manifest.group.as_str();

        if group != current_group {
            // Group header
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("  {}", group.to_uppercase()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])));
            current_group = group;
            render_idx += 1;
        }

        if logical_pos == app.cursor {
            list_state.select(Some(render_idx));
        }

        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                &script.manifest.name,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                &script.manifest.description,
                Style::default().fg(Color::DarkGray),
            ),
        ])));
        render_idx += 1;
    }

    if app.visible.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  Nenhum script disponível neste momento.",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let block = Block::default()
        .title(" agrr ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn key(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
}

fn desc(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::DarkGray))
}

fn render_status_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let right_line = Line::from(vec![
        key("↑↓"), desc(" navegar  "),
        key("ENTER"), desc(" executar  "),
        key("q"), desc(" sair"),
    ]);
    let right_width = 36u16;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(right_width + 2)])
        .split(area);

    let left_block = Block::default().borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP);
    let right_block = Block::default().borders(Borders::RIGHT | Borders::BOTTOM | Borders::TOP);

    let left_line = Line::from(vec![
        desc(" pressione "),
        key("/"),
        desc(" para buscar"),
    ]);

    let left_para = Paragraph::new(left_line).block(left_block);
    let right_para = Paragraph::new(right_line)
        .block(right_block)
        .alignment(Alignment::Right);

    frame.render_widget(left_para, chunks[0]);
    frame.render_widget(right_para, chunks[1]);
}

fn render_search_input(frame: &mut Frame, app: &App, area: Rect) {
    let query_display = format!(" {}_", app.search_query);
    let block = Block::default()
        .title(" Buscar ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let para = Paragraph::new(query_display)
        .block(block)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(para, area);
}

fn render_search_hints(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        desc("  "),
        key("ESC"), desc(" cancelar   "),
        key("↑↓"), desc(" navegar   "),
        key("ENTER"), desc(" executar"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
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

    let text = if arg.options.is_empty() {
        format!("{}: {}_", arg.prompt, current_input)
    } else {
        format!(
            "{} [{}]: {}_",
            arg.prompt,
            arg.options.join("/"),
            current_input
        )
    };

    render_centered_prompt(frame, &format!("  {}  ", script.manifest.name), &text);
}

fn render_cred_prompt(frame: &mut Frame, app: &App, key: &str) {
    let Mode::CollectingCred { script_idx, .. } = &app.mode else {
        return;
    };
    let script = &app.registry[*script_idx];
    let text = format!("{} para '{}': [entrada mascarada]_", key, script.manifest.name);
    render_centered_prompt(frame, "  Credenciais necessárias  ", &text);
}

fn render_ask_save(frame: &mut Frame, _app: &App, key: &str) {
    let text = format!(
        "Salvar credencial '{}' para próximas execuções? [s/N]",
        key
    );
    render_centered_prompt(frame, "  Salvar credencial?  ", &text);
}

fn render_centered_prompt(frame: &mut Frame, title: &str, text: &str) {
    let area = centered_rect(60, 20, frame.area());
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

// ─── Output pane ──────────────────────────────────────────────────────────────

fn render_output(frame: &mut Frame, app: &App, exit_code: Option<i32>, elapsed_ms: u64) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .output_lines
        .iter()
        .map(|l: &StyledLine| {
            let style = if l.is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(&l.text, style)))
        })
        .collect();

    let title = match exit_code {
        None => " Executando... ".into(),
        Some(0) => format!(" Concluído ✓ ({:.1}s) ", elapsed_ms as f64 / 1000.0),
        Some(c) => format!(" Concluído (código {} | {:.1}s) ", c, elapsed_ms as f64 / 1000.0),
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(if exit_code == Some(0) {
            Style::default().fg(Color::Green)
        } else if exit_code.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Blue)
        });

    let list = List::new(items).block(block);
    frame.render_widget(list, chunks[0]);

    if exit_code.is_some() {
        let hint = Paragraph::new("Pressione qualquer tecla para voltar ao menu")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[1]);
    }
}

// ─── Auth error ───────────────────────────────────────────────────────────────

fn render_auth_error(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 25, frame.area());
    let text = "Credenciais inválidas. As credenciais salvas foram removidas.\n\nDeseja tentar novamente com novas credenciais? [S/n]";
    let block = Block::default()
        .title("  Erro de autenticação  ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let para = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
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
