use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use pulldown_cmark::{Event, Options, Parser, Tag};
use std::path::Path;

use crate::{app::App, browser::Browser};

const PROGRESS_BAR_WIDTH: usize = 5;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let footer = footer_paragraph(
        if app.revealed {
            " ENTER: Hide answer  |  ↑/↓: Scroll  |  N: Next  |  P: Previous  |  Q: Quit "
        } else {
            " ENTER: Reveal answer  |  N: Next  |  P: Previous  |  R: Random  |  Q: Quit "
        },
        Style::default().fg(Color::Gray),
    );

    let footer_lines = footer.line_count(area.width) as u16;
    let footer_height = reserved_height(footer_lines, area.height);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(footer_height),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " FLASHCARDSTUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(header, layout[0]);

    let body = layout[1];
    let inner_width = body.width.saturating_sub(2);

    let (paragraph, content_height) = if app.revealed {
        let answer = &app.cards[app.current].answer;
        let text = if answer.trim().is_empty() {
            Text::from(Line::styled(
                "(no answer)",
                Style::default().fg(Color::Gray),
            ))
        } else {
            markdown_to_text(answer)
        };

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let height = paragraph.line_count(inner_width);
        clamp_scroll(app, height, body.height);
        (paragraph.scroll((app.scroll, 0)), height)
    } else {
        let paragraph = Paragraph::new(app.cards[app.current].question.as_str())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: true });

        let height = paragraph.line_count(inner_width);
        (paragraph, height)
    };

    // Card information with progress
    // shortcut: the bar has five cells, so it only approximates; the label is exact.
    let percent = (app.current + 1) * 100 / app.cards.len();
    let filled = ((app.current + 1) * PROGRESS_BAR_WIDTH).div_ceil(app.cards.len());
    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(PROGRESS_BAR_WIDTH - filled)
    );

    let title = Line::from(vec![
        Span::raw(format!(
            "{}  ·  {}/{}  ",
            app.filename,
            app.current + 1,
            app.cards.len()
        )),
        Span::styled(bar, Style::default().fg(Color::Cyan)),
        Span::raw(format!(" {percent}% ")),
    ]);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    // Frame hugs the text; taller content fills the body and scrolls.
    let card_area = centered_area(body, content_height + 2);
    let inner = block.inner(card_area);

    frame.render_widget(block, card_area);
    frame.render_widget(paragraph, inner);

    let footer_area = centered_area(layout[2], footer_lines as usize);
    frame.render_widget(footer, footer_area);

    if app.show_help && !draw_help(frame) {
        app.show_help = false;
    }
}

pub fn draw_browser(frame: &mut Frame, browser: &Browser) {
    let area = frame.area();

    let footer = match browser.message() {
        Some(message) => footer_paragraph(message, Style::default().fg(Color::Red)),
        None => footer_paragraph(
            " ↑/↓: Move  |  ENTER: Open  |  BACKSPACE: Up  |  Q: Quit ",
            Style::default().fg(Color::Gray),
        ),
    };

    let footer_lines = footer.line_count(area.width) as u16;
    let footer_height = reserved_height(footer_lines, area.height);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(footer_height)])
        .split(area);

    let block = Block::default()
        .title(format!(" Select deck · {} ", browser.cwd().display()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(layout[0]);
    frame.render_widget(block, layout[0]);

    if browser.entries().is_empty() {
        frame.render_widget(
            Paragraph::new("No folders or .md files here. Press BACKSPACE to go up.")
                .alignment(Alignment::Center),
            inner,
        );
    } else {
        // Keep the selection visible when the list is longer than the screen.
        let visible = inner.height as usize;
        let start = browser.selected.saturating_sub(visible.saturating_sub(1));

        let lines: Vec<Line> = browser
            .entries()
            .iter()
            .enumerate()
            .skip(start)
            .map(|(index, entry)| {
                let (name, is_dir) = match entry {
                    crate::browser::Entry::Dir(path) => (format!(" {}/", file_name(path)), true),
                    crate::browser::Entry::File(path) => (format!(" {}", file_name(path)), false),
                };

                let style = if index == browser.selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if is_dir {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                Line::from(Span::styled(name, style))
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), inner);
    }

    let footer_area = centered_area(layout[1], footer_lines as usize);
    frame.render_widget(footer, footer_area);
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn draw_help(frame: &mut Frame) -> bool {
    let area = frame.area();
    let width = area.width.saturating_sub(8).min(56);
    let height = area.height.saturating_sub(4).min(18);

    if width < 20 || height < 5 {
        return false;
    }

    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keybinds ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let key = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let section = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let lines = vec![
        Line::from(vec![
            Span::styled("ENTER       ", key),
            Span::raw("Reveal / hide answer"),
        ]),
        Line::from(vec![
            Span::styled("N / →       ", key),
            Span::raw("Next card"),
        ]),
        Line::from(vec![
            Span::styled("P / ←       ", key),
            Span::raw("Previous card"),
        ]),
        Line::from(vec![Span::styled("R           ", key), Span::raw("Random card")]),
        Line::from(vec![
            Span::styled("↑ / K       ", key),
            Span::raw("Scroll answer up"),
        ]),
        Line::from(vec![
            Span::styled("↓ / J       ", key),
            Span::raw("Scroll answer down"),
        ]),
        Line::from(vec![
            Span::styled("H / ESC     ", key),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![Span::styled("Q           ", key), Span::raw("Quit")]),
        Line::from(""),
        Line::from(Span::styled("File picker", section)),
        Line::from(vec![
            Span::styled("↑ / ↓       ", key),
            Span::raw("Move selection"),
        ]),
        Line::from(vec![
            Span::styled("ENTER / →   ", key),
            Span::raw("Open folder / choose deck"),
        ]),
        Line::from(vec![
            Span::styled("← / BKSP    ", key),
            Span::raw("Go up one folder"),
        ]),
        Line::from(vec![
            Span::styled("Q / ESC     ", key),
            Span::raw("Quit"),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);

    true
}

fn footer_paragraph<'a>(text: &'a str, style: Style) -> Paragraph<'a> {
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(style)
        .wrap(Wrap { trim: true })
}

// One blank row above the footer; shrink if the terminal is very short.
fn reserved_height(lines: u16, area_height: u16) -> u16 {
    (lines + 1).min(area_height.saturating_sub(4))
}

fn clamp_scroll(app: &mut App, content_height: usize, body_height: u16) {
    let visible = (content_height + 2)
        .min(body_height as usize)
        .saturating_sub(2);
    let max_scroll = content_height.saturating_sub(visible) as u16;

    app.scroll = app.scroll.min(max_scroll);
}

// Center a band of the given height inside `inner`; taller content fills `inner`.
fn centered_area(inner: Rect, height: usize) -> Rect {
    let height = height.min(inner.height as usize) as u16;

    Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints([Constraint::Length(height)])
        .split(inner)[0]
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
                    Tag::Heading { .. }
                        if !spans.is_empty() => {
                            lines.push(Line::from(std::mem::take(&mut spans)));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::App, browser::Browser, parser::Flashcard};
    use ratatui::{Terminal, backend::TestBackend};
    use std::fs;

    fn render(width: u16, height: u16, revealed: bool) -> Vec<String> {
        render_with(width, height, "Question", "Answer", revealed)
    }

    fn render_with(
        width: u16,
        height: u16,
        question: &str,
        answer: &str,
        revealed: bool,
    ) -> Vec<String> {
        render_cards(
            width,
            height,
            vec![Flashcard {
                question: question.to_string(),
                answer: answer.to_string(),
            }],
            revealed,
            false,
        )
    }

    fn render_cards(
        width: u16,
        height: u16,
        cards: Vec<Flashcard>,
        revealed: bool,
        show_help: bool,
    ) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();

        let mut app = App::new(cards, "test.md".to_string());
        app.revealed = revealed;
        app.show_help = show_help;

        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        rows(terminal.backend().buffer(), width, height)
    }

    fn rows(buffer: &ratatui::buffer::Buffer, width: u16, height: u16) -> Vec<String> {
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    fn card_bounds(lines: &[String]) -> (usize, usize) {
        let top = lines.iter().position(|l| l.contains('╭')).unwrap();
        let bottom = lines.iter().position(|l| l.contains('╰')).unwrap();
        (top, bottom)
    }

    fn assert_centered(text: &str, width: u16, height: u16, revealed: bool) {
        let lines = render(width, height, revealed);

        let row = lines
            .iter()
            .position(|line| line.contains(text))
            .unwrap_or_else(|| panic!("{text} not rendered at {width}x{height}"));
        let (top, bottom) = card_bounds(&lines);

        assert!(
            (row - top).abs_diff(bottom - row) <= 1,
            "{text} vertical at {width}x{height}: row {row}, card {top}..{bottom}"
        );

        let first = text.chars().next().unwrap();
        let col = lines[row].chars().position(|c| c == first).unwrap();
        let expected_col = width as usize / 2 - text.len() / 2;
        assert_eq!(col, expected_col, "{text} horizontal at {width}x{height}");
    }

    #[test]
    fn question_centered() {
        for (width, height) in [(80, 24), (40, 14), (30, 20), (120, 40)] {
            assert_centered("Question", width, height, false);
        }
    }

    #[test]
    fn answer_centered() {
        for (width, height) in [(80, 24), (40, 14), (30, 20), (120, 40)] {
            assert_centered("Answer", width, height, true);
        }
    }

    #[test]
    fn card_hugs_content() {
        let lines = render(80, 24, false);
        let (top, bottom) = card_bounds(&lines);
        assert_eq!(bottom - top + 1, 3, "one content line plus two borders");

        let lines = render_with(80, 24, "Q", "first\nsecond", true);
        let (top, bottom) = card_bounds(&lines);
        assert_eq!(bottom - top + 1, 4, "two content lines plus two borders");
    }

    #[test]
    fn card_fills_body_when_content_overflows() {
        let answer = "line\n\n".repeat(40);
        let lines = render_with(80, 24, "Q", &answer, true);

        let (top, bottom) = card_bounds(&lines);
        assert!(bottom - top + 1 > 10, "card grows with long answers");
    }

    #[test]
    fn footer_wraps_on_narrow_terminals() {
        let lines = render(30, 24, false);

        for label in ["ENTER", "Next", "Previous", "Random", "Quit"] {
            assert!(
                lines.iter().any(|line| line.contains(label)),
                "{label} missing at 30 wide"
            );
        }

        let footer_rows = lines
            .iter()
            .filter(|line| {
                ["ENTER", "Next", "Previous", "Random", "Quit"]
                    .iter()
                    .any(|label| line.contains(label))
            })
            .count();
        assert!(footer_rows > 1, "footer wraps into several rows");
    }

    #[test]
    fn title_shows_progress_bar_and_percentage() {
        let cards = (0..5)
            .map(|i| Flashcard {
                question: format!("Question {i}"),
                answer: "Answer".to_string(),
            })
            .collect();

        let lines = render_cards(80, 24, cards, false, false);
        let title = lines
            .iter()
            .find(|line| line.contains("test.md"))
            .expect("card title rendered");

        assert!(title.contains("1/5"), "position shown: {title}");
        assert!(title.contains("20%"), "percentage shown: {title}");
        assert_eq!(title.matches('█').count(), 1, "one of five cells filled");
        assert_eq!(title.matches('░').count(), 4, "four empty cells");
    }

    #[test]
    fn scroll_is_clamped_to_content() {
        let answer = "line\n\n".repeat(40);
        let mut app = App::new(
            vec![Flashcard {
                question: "Q".to_string(),
                answer,
            }],
            "test.md".to_string(),
        );
        app.revealed = true;
        app.scroll = u16::MAX;

        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        assert!(app.scroll > 0, "long answers keep some scroll");
        assert!(app.scroll < 40, "scroll stops at the end of the text");
    }

    #[test]
    fn help_state_clears_when_popup_cannot_fit() {
        let mut app = App::new(
            vec![Flashcard {
                question: "Question".to_string(),
                answer: "Answer".to_string(),
            }],
            "test.md".to_string(),
        );
        app.show_help = true;

        let mut terminal = Terminal::new(TestBackend::new(15, 6)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();

        assert!(!app.show_help, "invisible help must not swallow keys");
    }

    #[test]
    fn empty_answer_shows_a_notice() {
        let lines = render_with(80, 24, "Q", "   ", true);

        assert!(lines.iter().any(|line| line.contains("(no answer)")));
    }

    #[test]
    fn progress_percentage_is_exact_position() {
        let cards = (0..15)
            .map(|i| Flashcard {
                question: format!("Question {i}"),
                answer: "Answer".to_string(),
            })
            .collect();

        let lines = render_cards(80, 24, cards, false, false);
        let title = lines
            .iter()
            .find(|line| line.contains("test.md"))
            .expect("card title rendered");

        assert!(title.contains("1/15"));
        assert!(title.contains("6%"), "exact percentage: {title}");
        assert_eq!(title.matches('█').count(), 1, "bar is approximate");
    }

    #[test]
    fn help_overlay_lists_keybinds() {
        let lines = render_cards(
            80,
            24,
            vec![Flashcard {
                question: "Question".to_string(),
                answer: "Answer".to_string(),
            }],
            false,
            true,
        );

        for text in [
            "Keybinds",
            "Reveal / hide answer",
            "Next card",
            "Random card",
            "Toggle this help",
            "File picker",
            "choose deck",
            "Quit",
        ] {
            assert!(
                lines.iter().any(|line| line.contains(text)),
                "{text} missing from help screen"
            );
        }
    }

    #[test]
    fn browser_lists_directories_and_markdown_files() {
        let dir = std::env::temp_dir()
            .join(format!("flashcards-ui-browser-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("decks")).unwrap();
        fs::write(dir.join("bio.md"), "# Q\nA").unwrap();

        let browser = Browser::new(dir).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();

        terminal
            .draw(|frame| draw_browser(frame, &browser))
            .unwrap();

        let lines = rows(terminal.backend().buffer(), 60, 10);

        assert!(lines.iter().any(|line| line.contains("decks/")));
        assert!(lines.iter().any(|line| line.contains("bio.md")));
        assert!(lines.iter().any(|line| line.contains("Select deck")));
    }

    #[test]
    fn footer_stays_one_line_on_wide_terminals() {
        let lines = render(120, 40, false);

        let footer_rows = lines
            .iter()
            .filter(|line| {
                ["ENTER", "Next", "Previous", "Random", "Quit"]
                    .iter()
                    .any(|label| line.contains(label))
            })
            .count();
        assert_eq!(footer_rows, 1);
    }
}


