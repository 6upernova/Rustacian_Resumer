use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use super::app::{App, View};
use super::crab;

const ACCENT: Color = Color::LightRed;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();

    if size.width < 60 || size.height < 14 {
        let msg = Paragraph::new(
            "Terminal muy pequeño para la TUI.\n\nAumentá el tamaño de la ventana y reintentá.",
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(msg, size);
        return;
    }

    let left_width = crab::max_width().saturating_add(2).max(18);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left_width), Constraint::Min(0)])
        .split(size);

    render_crab(f, cols[0], app);
    render_right(f, cols[1], app);
}

fn render_crab<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let frame = crab::frame(app.crab_frame);

    // Neofetch style: ASCII a la izquierda, sin demasiada ornamentación.
    let widget = Paragraph::new(frame)
        .style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left);

    f.render_widget(widget, area);
}

fn render_right<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(area);

    render_header(f, rows[0], app);

    match &app.view {
        View::Selector => render_selector(f, rows[1], app),
        View::Processing { topic } => render_processing(f, rows[1], &topic.name),
        View::Summary { topic, report } => render_summary(f, rows[1], &topic.name, report),
        View::Error { message } => render_error(f, rows[1], message),
        View::Empty { message } => render_empty(f, rows[1], message),
    }
}

fn render_header<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let text = vec![
        Line::from(Span::styled(
            "Rustacian Resumer",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Base: {}", app.docs_dir.display())),
    ];

    let header = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Resumidor TF‑IDF "))
        .wrap(Wrap { trim: true });

    f.render_widget(header, area);
}

fn render_selector<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let mut items = app
        .topics
        .iter()
        .map(|t| ListItem::new(t.name.clone()))
        .collect::<Vec<_>>();
    items.push(ListItem::new("Salir"));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Tópicos "))
        .highlight_style(
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected.min(app.menu_len().saturating_sub(1))));

    f.render_stateful_widget(list, rows[0], &mut state);

    let help = Paragraph::new("↑/↓ mover · Enter seleccionar · q salir")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Controles "))
        .alignment(Alignment::Center);

    f.render_widget(help, rows[1]);
}

fn render_processing<B: Backend>(f: &mut Frame<B>, area: Rect, topic_name: &str) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let body = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "Procesando…",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Tópico: {topic_name}")),
        Line::from(""),
        Line::from("Leyendo .txt y calculando TF‑IDF."),
        Line::from("Esto puede tardar unos segundos."),
    ]))
    .wrap(Wrap { trim: false })
    .block(Block::default().borders(Borders::ALL).title(" Estado "));

    f.render_widget(body, rows[0]);

    let help = Paragraph::new("q salir")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Controles "))
        .alignment(Alignment::Center);

    f.render_widget(help, rows[1]);
}

fn render_summary<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    topic_name: &str,
    report: &crate::summarizer::SummaryReport,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "Tópico: {topic_name} · Archivos: {} · Oraciones: {}",
            report.file_count,
            report.top_sentences.len()
        ),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (i, s) in report.top_sentences.iter().enumerate() {
        lines.push(Line::from(format!("{:>2}. {}", i + 1, s.raw)));
        lines.push(Line::from(""));
    }

    if !report.warnings.is_empty() {
        lines.push(Line::from(Span::styled(
            "Warnings:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        for w in &report.warnings {
            lines.push(Line::from(format!("- {w}")));
        }
    }

    let body = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(" Resumen "));

    f.render_widget(body, rows[0]);

    let help = Paragraph::new("Esc/b volver · q salir")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Controles "))
        .alignment(Alignment::Center);

    f.render_widget(help, rows[1]);
}

fn render_error<B: Backend>(f: &mut Frame<B>, area: Rect, message: &str) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let body = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            "Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message),
    ]))
    .wrap(Wrap { trim: false })
    .block(Block::default().borders(Borders::ALL).title(" Estado "));

    f.render_widget(body, rows[0]);

    let help = Paragraph::new("Esc/b volver · q salir")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Controles "))
        .alignment(Alignment::Center);

    f.render_widget(help, rows[1]);
}

fn render_empty<B: Backend>(f: &mut Frame<B>, area: Rect, message: &str) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let body = Paragraph::new(message)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(" Sin tópicos "));

    f.render_widget(body, rows[0]);

    let help = Paragraph::new("q salir · Enter salir")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Controles "))
        .alignment(Alignment::Center);

    f.render_widget(help, rows[1]);
}
