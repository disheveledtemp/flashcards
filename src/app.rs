use crate::parser::Flashcard;
use rand::Rng;

pub struct App {
    pub cards: Vec<Flashcard>,
    pub current: usize,
    pub revealed: bool,
    pub scroll: u16,
    pub filename: String,
    pub show_help: bool,
}

impl App {
    pub fn new(cards: Vec<Flashcard>, filename: String) -> Self {
        Self {
            cards,
            current: 0,
            revealed: false,
            scroll: 0,
            filename,
            show_help: false,
        }
    }

    pub fn next(&mut self) {
        if self.current + 1 < self.cards.len() {
            self.current += 1;
        } else {
            self.current = 0;
        }

        self.reset_view();
    }

    pub fn previous(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        } else {
            self.current = self.cards.len() - 1;
        }

        self.reset_view();
    }

    pub fn random(&mut self) {
        if self.cards.len() > 1 {
            let mut rng = rand::rng();

            let mut next = rng.random_range(0..self.cards.len());

            while next == self.current {
                next = rng.random_range(0..self.cards.len());
            }

            self.current = next;
        }

        self.reset_view();
    }

    pub fn toggle_answer(&mut self) {
        self.revealed = !self.revealed;
        self.scroll = 0;
    }

    fn reset_view(&mut self) {
        self.revealed = false;
        self.scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Flashcard;

    fn app(count: usize) -> App {
        let cards = (0..count)
            .map(|i| Flashcard {
                question: format!("Q{i}"),
                answer: format!("A{i}"),
            })
            .collect();

        App::new(cards, "test.md".to_string())
    }

    #[test]
    fn next_and_previous_wrap() {
        let mut app = app(2);

        app.next();
        assert_eq!(app.current, 1);
        app.next();
        assert_eq!(app.current, 0);
        app.previous();
        assert_eq!(app.current, 1);
    }

    #[test]
    fn random_never_picks_the_current_card() {
        let mut app = app(2);

        for _ in 0..20 {
            let before = app.current;
            app.random();
            assert_ne!(app.current, before);
        }
    }

    #[test]
    fn moving_cards_resets_the_view() {
        let mut app = app(2);
        app.toggle_answer();
        app.scroll = 7;

        app.next();

        assert!(!app.revealed);
        assert_eq!(app.scroll, 0);
    }
}
