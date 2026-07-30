use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterState {
    #[allow(dead_code)]
    Pending,
    Correct,
    Misplaced,
    Absent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Won,
    Lost,
}

pub const MAX_ATTEMPTS: usize = 6;
pub const WORD_LENGTH: usize = 5;

pub struct Game {
    pub answer: Vec<char>,
    pub board: [[Option<char>; WORD_LENGTH]; MAX_ATTEMPTS],
    pub results: [[Option<LetterState>; WORD_LENGTH]; MAX_ATTEMPTS],
    pub current_row: usize,
    pub current_col: usize,
    pub phase: Phase,
}

impl Game {
    pub fn new(answer: String) -> Self {
        let chars: Vec<char> = answer.chars().collect();
        assert_eq!(chars.len(), WORD_LENGTH);

        Self {
            answer: chars,
            board: [[None; WORD_LENGTH]; MAX_ATTEMPTS],
            results: [[None; WORD_LENGTH]; MAX_ATTEMPTS],
            current_row: 0,
            current_col: 0,
            phase: Phase::Playing,
        }
    }

    pub fn type_letter(&mut self, c: char) -> Result<(), ()> {
        if self.phase != Phase::Playing {
            return Err(());
        }
        if self.current_col >= WORD_LENGTH {
            return Err(());
        }
        self.board[self.current_row][self.current_col] = Some(c);
        self.current_col += 1;
        Ok(())
    }

    pub fn delete_letter(&mut self) {
        if self.current_col == 0 {
            return;
        }
        self.current_col -= 1;
        self.board[self.current_row][self.current_col] = None;
    }

    pub fn submit_guess(&mut self) -> Result<[LetterState; WORD_LENGTH], &'static str> {
        if self.phase != Phase::Playing {
            return Err("Game is over");
        }
        if self.current_col < WORD_LENGTH {
            return Err("Not enough letters");
        }

        let mut guess = Vec::with_capacity(WORD_LENGTH);
        for i in 0..WORD_LENGTH {
            guess.push(self.board[self.current_row][i].unwrap());
        }

        let result = evaluate_guess(&guess, &self.answer);
        self.results[self.current_row] = result.map(|s| Some(s));

        self.current_row += 1;
        self.current_col = 0;

        if result.iter().all(|&s| s == LetterState::Correct) {
            self.phase = Phase::Won;
        } else if self.current_row >= MAX_ATTEMPTS {
            self.phase = Phase::Lost;
        }

        Ok(result)
    }

    pub fn keyboard_states(&self) -> HashMap<char, LetterState> {
        let mut states: HashMap<char, LetterState> = HashMap::new();

        for row in 0..self.current_row {
            for col in 0..WORD_LENGTH {
                if let Some(c) = self.board[row][col] {
                    if let Some(Some(state)) = self.results.get(row).map(|r| &r[col]) {
                        let current = states.get(&c).copied();
                        let best = match (current, state) {
                            (None, _) => *state,
                            (Some(LetterState::Correct), _) => LetterState::Correct,
                            (Some(LetterState::Misplaced), LetterState::Absent) => LetterState::Misplaced,
                            (Some(LetterState::Absent), LetterState::Misplaced) => LetterState::Misplaced,
                            _ => *state,
                        };
                        states.insert(c, best);
                    }
                }
            }
        }

        states
    }

    pub fn current_guess_string(&self) -> String {
        self.board[self.current_row][..self.current_col]
            .iter()
            .map(|c| c.unwrap_or(' '))
            .collect()
    }

    pub fn answer_string(&self) -> String {
        self.answer.iter().collect()
    }

    pub fn guesses(&self) -> Vec<String> {
        self.board[..self.current_row]
            .iter()
            .map(|row| row.iter().map(|c| c.unwrap_or(' ')).collect())
            .collect()
    }

    pub fn restore_with_guesses(answer: String, guesses: &[String]) -> Self {
        let mut game = Game::new(answer);
        for guess in guesses {
            for c in guess.chars() {
                let _ = game.type_letter(c);
            }
            let _ = game.submit_guess();
        }
        game
    }
}

fn evaluate_guess(guess: &[char], answer: &[char]) -> [LetterState; WORD_LENGTH] {
    let mut result = [LetterState::Absent; WORD_LENGTH];
    let mut answer_used = [false; WORD_LENGTH];

    for i in 0..WORD_LENGTH {
        if guess[i] == answer[i] {
            result[i] = LetterState::Correct;
            answer_used[i] = true;
        }
    }

    for i in 0..WORD_LENGTH {
        if result[i] == LetterState::Correct {
            continue;
        }
        for j in 0..WORD_LENGTH {
            if !answer_used[j] && guess[i] == answer[j] {
                result[i] = LetterState::Misplaced;
                answer_used[j] = true;
                break;
            }
        }
    }

    result
}
