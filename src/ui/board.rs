use crate::game::state::Game;
use gtk::prelude::*;
use gtk::Grid;

use crate::game::state::{LetterState, MAX_ATTEMPTS, WORD_LENGTH};
use crate::ui::tile::Tile;

pub struct Board {
    pub grid: Grid,
    tiles: Vec<Vec<Tile>>,
}

impl Board {
    pub fn new() -> Self {
        let grid = Grid::builder()
            .css_classes(["board-grid"])
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .row_spacing(4)
            .column_spacing(4)
            .build();

        let mut tiles = Vec::new();
        for row in 0..MAX_ATTEMPTS {
            let mut row_tiles = Vec::new();
            for col in 0..WORD_LENGTH {
                let tile = Tile::new();
                grid.attach(tile.widget(), col as i32, row as i32, 1, 1);
                row_tiles.push(tile);
            }
            tiles.push(row_tiles);
        }

        Self { grid, tiles }
    }

    pub fn set_letter(&self, row: usize, col: usize, c: char) {
        if row < MAX_ATTEMPTS && col < WORD_LENGTH {
            self.tiles[row][col].set_letter(c);
        }
    }

    pub fn clear_letter(&self, row: usize, col: usize) {
        if row < MAX_ATTEMPTS && col < WORD_LENGTH {
            self.tiles[row][col].clear();
        }
    }

    pub fn reveal_row(&self, row: usize, result: &[LetterState; WORD_LENGTH]) {
        for (col, &state) in result.iter().enumerate() {
            self.tiles[row][col].set_state(state);
        }
    }

    pub fn restore_from_game(&self, game: &Game) {
        for row in 0..game.current_row {
            for col in 0..WORD_LENGTH {
                if let Some(c) = game.board[row][col] {
                    self.tiles[row][col].set_letter(c);
                }
                if let Some(state) = game.results[row][col] {
                    self.tiles[row][col].set_state(state);
                }
            }
        }
    }

    pub fn reset(&self) {
        for row in 0..MAX_ATTEMPTS {
            for col in 0..WORD_LENGTH {
                self.tiles[row][col].clear();
            }
        }
    }

    pub fn widget(&self) -> &Grid {
        &self.grid
    }
}
