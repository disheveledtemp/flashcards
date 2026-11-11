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

    let mut cards = Vec::new();
    let mut question = String::new();
    let mut answer = String::new();

    let mut inside_card = false;
    let mut fence: Option<char> = None;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim();

        // Detect fenced code blocks.
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