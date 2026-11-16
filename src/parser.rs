use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Flashcard {
    pub question: String,
    pub answer: String,
}

pub fn load_flashcards(path: &Path) -> Result<Vec<Flashcard>> {
    let content = fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    let content = content.trim_start_matches('\u{feff}');

    let mut cards = Vec::new();
    let mut question = String::new();
    let mut answer = String::new();

    let mut inside_card = false;
    let mut fence: Option<char> = None;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim();

        // Detect fenced code blocks.
        // shortcut: an unclosed fence swallows every later card; close the fence
        // or split the deck if that ever becomes a real problem.
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let marker = trimmed.chars().next().unwrap();

            if fence == Some(marker) {
                fence = None;
            } else if fence.is_none() {
                fence = Some(marker);
            }

            if inside_card {
                answer.push_str(line);
            }

            continue;
        }

        // Ignore headings inside code blocks.
        if fence.is_some() {
            if inside_card {
                answer.push_str(line);
            }
            continue;
        }

        // Detect H1 headings only.
        let is_h1 = trimmed == "#"
            || trimmed.starts_with("# ");

        if is_h1 {
            // Save previous card.
            if inside_card {
                cards.push(Flashcard {
                    question: question.trim().to_string(),
                    answer: answer.trim().to_string(),
                });
            }

            question = trimmed
                .strip_prefix('#')
                .unwrap()
                .trim()
                .to_string();

            answer.clear();
            inside_card = true;
        } else if inside_card {
            answer.push_str(line);
        }
    }

    // Save final card.
    if inside_card {
        cards.push(Flashcard {
            question: question.trim().to_string(),
            answer: answer.trim().to_string(),
        });
    }

    if cards.is_empty() {
        anyhow::bail!("No H1 headings found in the Markdown file.");
    }

    Ok(cards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static NEXT: AtomicU32 = AtomicU32::new(0);

    fn load(content: &str) -> Result<Vec<Flashcard>> {
        let path = std::env::temp_dir().join(format!(
            "flashcards-parser-{}-{}.md",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, content).unwrap();
        let cards = load_flashcards(&path);
        let _ = fs::remove_file(&path);
        cards
    }

    #[test]
    fn splits_cards_on_h1_headings() {
        let cards = load("# One\nA1\n\n# Two\nA2\n").unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].question, "One");
        assert_eq!(cards[0].answer, "A1");
        assert_eq!(cards[1].question, "Two");
        assert_eq!(cards[1].answer, "A2");
    }

    #[test]
    fn loads_deck_with_utf8_bom() {
        let cards = load("\u{feff}# One\nA1\n").unwrap();

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].question, "One");
    }

    #[test]
    fn loads_crlf_deck() {
        let cards = load("# One\r\nA1\r\n\r\n# Two\r\nA2\r\n").unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[1].question, "Two");
    }

    #[test]
    fn heading_inside_fence_stays_in_answer() {
        let cards = load("# One\n```\n# not a card\n```\n# Two\nA2\n").unwrap();

        assert_eq!(cards.len(), 2);
        assert!(cards[0].answer.contains("# not a card"));
    }

    #[test]
    fn unclosed_fence_swallows_later_cards() {
        let cards = load("# One\nA1\n```\n# Two\nA2\n").unwrap();

        assert_eq!(cards.len(), 1);
    }

    #[test]
    fn no_headings_is_an_error() {
        assert!(load("just text\n").is_err());
    }
}