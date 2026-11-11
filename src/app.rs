use crate::parser::Flashcard;
use rand::Rng;

pub struct App {
    pub cards: Vec<Flashcard>,
    pub current: usize,
    pub revealed: bool,
    pub scroll: u16,
    pub filename: String,
    pub running: bool,
}

impl App {
    pub fn new(cards: Vec<Flashcard>, filename: String) -> Self {
        Self {
            cards,
            current: 0,
            revealed: false,
            scroll: 0,
            filename,
            running: true,
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
