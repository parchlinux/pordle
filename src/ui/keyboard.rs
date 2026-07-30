use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Button, Grid};

use crate::game::state::LetterState;

const KEYBOARD_ROWS: &[&[&str]] = &[
    &["ا", "ب", "پ", "ت", "ث", "ج", "چ", "ح", "خ"],
    &["د", "ذ", "ر", "ز", "ژ", "س", "ش", "ص", "ض"],
    &["ط", "ظ", "ع", "غ", "ف", "ق", "ک", "گ", "ل"],
    &["م", "ن", "و", "ه", "ی"],
];

pub enum KeyEvent {
    Letter(char),
    Enter,
    Backspace,
}

pub struct Keyboard {
    pub grid: Grid,
    keys: HashMap<char, Button>,
    callback: Rc<RefCell<Option<Box<dyn Fn(KeyEvent)>>>>,
}

use std::cell::RefCell;

impl Keyboard {
    pub fn new() -> Self {
        let grid = Grid::builder()
            .css_classes(["keyboard-grid"])
            .halign(gtk::Align::Center)
            .row_spacing(4)
            .column_spacing(1)
            .build();

        let mut keys = HashMap::new();
        let callback: Rc<RefCell<Option<Box<dyn Fn(KeyEvent)>>>> = Rc::new(RefCell::new(None));

        let mut row_index = 0i32;

        for row in KEYBOARD_ROWS {
            let col_start = if row.len() < 9 { 2 } else { 0 };
            let mut col_index = col_start;

            for &letter_str in *row {
                let ch = letter_str.chars().next().unwrap();
                let button = Button::with_label(letter_str);
                button.set_css_classes(&["keyboard-key", "key-unused"]);
                button.set_size_request(40, -1);

                let cb = callback.clone();
                button.connect_clicked(move |_| {
                    if let Some(ref f) = *cb.borrow() {
                        f(KeyEvent::Letter(ch));
                    }
                });

                grid.attach(&button, col_index, row_index, 1, 1);
                keys.insert(ch, button);
                col_index += 1;
            }

            row_index += 1;
        }

        let last_row = (KEYBOARD_ROWS.len() - 1) as i32;

        let enter_button = Button::with_label("تأیید");
        enter_button.set_css_classes(&["keyboard-key", "special-key", "key-unused"]);
        enter_button.set_size_request(56, -1);

        let cb = callback.clone();
        enter_button.connect_clicked(move |_| {
            if let Some(ref f) = *cb.borrow() {
                f(KeyEvent::Enter);
            }
        });
        grid.attach(&enter_button, 0, last_row, 1, 1);

        let backspace_button = Button::with_label("⌫");
        backspace_button.set_css_classes(&["keyboard-key", "special-key", "key-unused"]);
        backspace_button.set_size_request(56, -1);

        let cb = callback.clone();
        backspace_button.connect_clicked(move |_| {
            if let Some(ref f) = *cb.borrow() {
                f(KeyEvent::Backspace);
            }
        });
        grid.attach(&backspace_button, 7, last_row, 1, 1);

        Self {
            grid,
            keys,
            callback,
        }
    }

    pub fn set_callback<F: Fn(KeyEvent) + 'static>(&self, f: F) {
        *self.callback.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_key_state(&self, c: char, state: LetterState) {
        if let Some(button) = self.keys.get(&c) {
            let class = match state {
                LetterState::Correct => "key-correct",
                LetterState::Misplaced => "key-misplaced",
                LetterState::Absent => "key-absent",
                LetterState::Pending => "key-unused",
            };
            button.set_css_classes(&["keyboard-key", class]);
        }
    }

    pub fn send_event(&self, event: KeyEvent) {
        if let Some(ref f) = *self.callback.borrow() {
            f(event);
        }
    }

    pub fn reset_states(&self) {
        for button in self.keys.values() {
            button.set_css_classes(&["keyboard-key", "key-unused"]);
        }
    }

    pub fn update_states(&self, states: &HashMap<char, LetterState>) {
        for (&c, &state) in states {
            self.set_key_state(c, state);
        }
    }

    pub fn widget(&self) -> &Grid {
        &self.grid
    }
}
