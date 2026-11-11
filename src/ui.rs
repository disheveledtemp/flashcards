use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use pulldown_cmark::{Event, Options, Parser, Tag};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " FLASHCARDS ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  STUDY MODE",
            Style::default().fg(Color::Green),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(header, layout[0]);

    let card = &app.cards[app.current];

    // Card information
    let title = format!(
        "{}  ·  {}/{}",
        app.filename,
        app.current + 1,
        app.cards.len()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(layout[1]);
    frame.render_widget(block, layout[1]);

    if app.revealed {
        let text = markdown_to_text(&card.answer);

        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((app.scroll, 0));

        frame.render_widget(paragraph, inner);
    } else {
        let question = Paragraph::new(card.question.as_str())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: true });

        let centered = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(3),
                Constraint::Percentage(65),
            ])
            .split(inner);

        frame.render_widget(question, centered[1]);
    }

    // Footer
    let footer = if app.revealed {
        " ENTER: Hide answer  |  ↑/↓: Scroll  |  N: Next  |  P: Previous  |  Q: Quit "
    } else {
        " ENTER: Reveal answer  |  N: Next  |  P: Previous  |  R: Random  |  Q: Quit "
    };

    let footer = Paragraph::new(footer)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(footer, layout[2]);
}

// Convert Markdown into terminal-formatted text.
fn markdown_to_text(markdown: &str) -> Text<'static> {
    let parser = Parser::new_ext(markdown, Options::all());

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();

    let mut styles = vec![Style::default()];

    for event in parser {
        match event {
            Event::Start(tag) => {
                let parent = *styles.last().unwrap();

                let style = match &tag {
                    Tag::Strong => parent.add_modifier(Modifier::BOLD),
                    Tag::Emphasis => parent.add_modifier(Modifier::ITALIC),
                    Tag::Strikethrough => parent.add_modifier(Modifier::CROSSED_OUT),
                    Tag::Heading { .. } => parent
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    Tag::CodeBlock(_) => parent.fg(Color::Yellow),
                    Tag::BlockQuote(_) => parent.fg(Color::Gray),
                    _ => parent,
                };

                match tag {
                    Tag::Paragraph => {
                        if !spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut spans)));
                        }
                    }
                    Tag::Item => {
                        spans.push(Span::raw("  • "));
                    }
                    Tag::BlockQuote(_) => {
                        spans.push(Span::raw("  │ "));
                    }
                    Tag::Heading { .. } => {
                        if !spans.is_empty() {
                            lines.push(Line::from(std::mem::take(&mut spans)));
                        }
                    }
                    _ => {}
                }

                styles.push(style);
            }

            Event::End(tag) => {
                styles.pop();

                match tag {
                    pulldown_cmark::TagEnd::Paragraph
                    | pulldown_cmark::TagEnd::Heading(_)
                    | pulldown_cmark::TagEnd::Item
                    | pulldown_cmark::TagEnd::CodeBlock => {
                        lines.push(Line::from(std::mem::take(&mut spans)));
                    }
                    _ => {}
                }
            }

            Event::Text(text) | Event::Code(text) => {
                spans.push(Span::styled(
                    text.to_string(),
                    *styles.last().unwrap(),
                ));
            }

            Event::SoftBreak | Event::HardBreak => {
                lines.push(Line::from(std::mem::take(&mut spans)));
            }

            Event::Rule => {
                lines.push(Line::from("────────────────────"));
            }

            Event::Html(html) | Event::InlineHtml(html) => {
                spans.push(Span::raw(html.to_string()));
            }

            _ => {}
        }
    }

    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }

    Text::from(lines)
}
