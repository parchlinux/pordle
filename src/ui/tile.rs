use gtk::prelude::*;
use gtk::Button;

use crate::game::state::LetterState;

pub struct Tile {
    pub button: Button,
}

impl Tile {
    pub fn new() -> Self {
        let button = Button::builder()
            .css_classes(["tile", "tile-empty"])
            .build();

        button.add_css_class("tile");

        Self { button }
    }

    pub fn set_letter(&self, c: char) {
        self.button.set_label(&c.to_string());
        self.button.add_css_class("tile-filling");
    }

    pub fn set_state(&self, state: LetterState) {
        let class = match state {
            LetterState::Correct => "tile-correct",
            LetterState::Misplaced => "tile-misplaced",
            LetterState::Absent => "tile-absent",
            LetterState::Pending => "tile-empty",
        };

        let classes = vec!["tile", class];
        self.button.set_css_classes(&classes);
    }

    pub fn clear(&self) {
        self.button.set_label("");
        self.button.set_css_classes(&["tile", "tile-empty"]);
    }

    pub fn widget(&self) -> &Button {
        &self.button
    }
}
