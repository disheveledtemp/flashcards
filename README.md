# flashcards

A terminal flashcard app. Study Markdown decks in your terminal.

## Install

```sh
cargo install --git https://github.com/disheveledtemp/flashcards
```

Requires a Rust toolchain (`rustup` is the usual way to get one: https://rustup.rs).

## Usage

```sh
# Open a specific deck
flashcards ~/notes/biology.md

# Or browse for a deck interactively
flashcards
```

## Deck format

Each H1 heading starts a new card. The heading is the question; everything below it, until the next H1, is the answer. Anything else in the file (H2 headings, lists, code fences, other Markdown) is part of the answer.

```markdown
# What is the powerhouse of the cell?

The mitochondria.

# What does DNA stand for?

Deoxyribonucleic acid.
```

## Keys

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit (browser) / close help (study) |
| `j`/`k`, arrows | Navigate |
| `Enter` | Open deck (browser) / reveal answer (study) |
| `h` | Help |
| `n` / `p` | Next / previous card |
| `r` | Jump to random card |

## License

MIT
