use crate::persian::{is_persian_letter, normalize_persian_str};
use rusqlite::{Connection, Result as SqlResult, params};
use std::path::PathBuf;
use std::fs;

const DB_FILENAME: &str = "pordle.db";
const APP_DIR: &str = "pordle";
const EMBEDDED_WORDS: &str = include_str!("../data/words.txt");


pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn open() -> SqlResult<Self> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_or_default() -> Self {
        Self::open().expect("Failed to open database")
    }

    fn db_path() -> PathBuf {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join(APP_DIR).join(DB_FILENAME)
        } else {
            PathBuf::from(".").join(DB_FILENAME)
        }
    }

    fn migrate(&mut self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS words (
                word TEXT PRIMARY KEY CHECK(length(word) = 5),
                added_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS stats (
                id INTEGER PRIMARY KEY CHECK(id = 1),
                games_played INTEGER NOT NULL DEFAULT 0,
                games_won INTEGER NOT NULL DEFAULT 0,
                current_streak INTEGER NOT NULL DEFAULT 0,
                max_streak INTEGER NOT NULL DEFAULT 0,
                guess_distribution TEXT NOT NULL DEFAULT '[0,0,0,0,0,0]'
            );

            CREATE TABLE IF NOT EXISTS daily_results (
                date TEXT PRIMARY KEY,
                answer TEXT NOT NULL,
                won INTEGER NOT NULL,
                guesses TEXT NOT NULL
            );

            INSERT OR IGNORE INTO stats (id) VALUES (1);"
        )?;
        Ok(())
    }

    pub fn add_word(&mut self, word: &str) -> Result<AddWordResult, rusqlite::Error> {
        let clean = normalize_persian_str(word.trim().trim_matches('\'').trim_matches('"'));

        if clean.chars().count() != 5 || !clean.chars().all(is_persian_letter) {
            return Ok(AddWordResult::Invalid);
        }

        let result = self.conn.execute(
            "INSERT OR IGNORE INTO words (word) VALUES (?1)",
            params![clean],
        )?;
        if result > 0 {
            Ok(AddWordResult::Added)
        } else {
            Ok(AddWordResult::Duplicate)
        }
    }

    pub fn is_valid_word(&self, word: &str) -> bool {
        let word_norm = normalize_persian_str(word);
        self.conn
            .query_row(
                "SELECT 1 FROM words WHERE word = ?1",
                params![word_norm],
                |_| Ok(()),
            )
            .is_ok()
    }

    pub fn word_count(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM words", [], |row| row.get::<_, usize>(0))
            .unwrap_or(0)
    }

    pub fn random_word(&self) -> Option<String> {
        self.conn
            .query_row("SELECT word FROM words ORDER BY RANDOM() LIMIT 1", [], |row| {
                row.get(0)
            })
            .ok()
            .map(|w: String| normalize_persian_str(&w))
    }

    pub fn daily_word(&self) -> Option<String> {
        let count = self.word_count();
        if count == 0 {
            return None;
        }
        let days = glib::DateTime::now_local()
            .map(|dt| {
                let utc_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                let offset_secs = dt.utc_offset().as_seconds();
                ((utc_secs + offset_secs) / 86400) as usize
            })
            .unwrap_or_else(|_| {
                let secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                (secs / 86400) as usize
            });
        let index = days % count;
        self.word_at_index(index)
    }

    fn word_at_index(&self, index: usize) -> Option<String> {
        self.conn
            .query_row(
                "SELECT word FROM words ORDER BY word ASC LIMIT 1 OFFSET ?1",
                params![index as i64],
                |row| row.get(0),
            )
            .ok()
            .map(|w: String| normalize_persian_str(&w))
    }

    pub fn import_from_file(&mut self, path: &str) -> SqlResult<(usize, usize)> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e))),
        };

        let mut total = 0;
        let mut added = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            for token in line.split(|c: char| c == ',' || c == '[' || c == ']') {
                let token = token.trim().trim_matches('\'').trim_matches('"');
                if token.is_empty() {
                    continue;
                }

                total += 1;
                if let Ok(AddWordResult::Added) = self.add_word(token) {
                    added += 1;
                }
            }
        }

        Ok((total, added))
    }

    pub fn import_from_string(&mut self, content: &str) -> (usize, usize) {
        let mut total = 0;
        let mut added = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            for token in line.split(|c: char| c == ',' || c == '[' || c == ']') {
                let token = token.trim().trim_matches('\'').trim_matches('"');
                if token.is_empty() {
                    continue;
                }

                total += 1;
                if let Ok(AddWordResult::Added) = self.add_word(token) {
                    added += 1;
                }
            }
        }

        (total, added)
    }

    pub fn populate_from_default_files(&mut self) {
        let word_count = self.word_count();
        if word_count > 0 {
            return;
        }

        let paths = [
            "words.txt",
            "/usr/share/pordle/words.txt",
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                if let Ok((total, added)) = self.import_from_file(path) {
                    eprintln!("Imported {}/{} words from {}", added, total, path);
                }
                if self.word_count() > 0 {
                    return;
                }
            }
        }

        // Fallback to compiled-in default word list
        let (total, added) = self.import_from_string(EMBEDDED_WORDS);
        eprintln!("Imported {}/{} default embedded words", added, total);
    }

    pub fn get_stats(&self) -> GameStats {
        let default = || GameStats {
            games_played: 0,
            games_won: 0,
            current_streak: 0,
            max_streak: 0,
            guess_distribution: [0; 6],
        };

        self.conn
            .query_row("SELECT games_played, games_won, current_streak, max_streak, guess_distribution FROM stats WHERE id = 1", 
                [], 
                |row| {
                    let dist_str: String = row.get(4)?;
                    let dist: [i32; 6] = serde_json::from_str(&dist_str).unwrap_or([0; 6]);
                    Ok(GameStats {
                        games_played: row.get(0)?,
                        games_won: row.get(1)?,
                        current_streak: row.get(2)?,
                        max_streak: row.get(3)?,
                        guess_distribution: dist,
                    })
                })
            .unwrap_or_else(|_| default())
    }

    pub fn record_game(&self, won: bool, guess_count: usize) -> SqlResult<()> {
        let mut stats = self.get_stats();
        stats.games_played += 1;

        if won {
            stats.games_won += 1;
            stats.current_streak += 1;
            if stats.current_streak > stats.max_streak {
                stats.max_streak = stats.current_streak;
            }
            if guess_count > 0 && guess_count <= 6 {
                stats.guess_distribution[guess_count - 1] += 1;
            }
        } else {
            stats.current_streak = 0;
        }

        let dist_json = serde_json::to_string(&stats.guess_distribution).unwrap();
        self.conn.execute(
            "UPDATE stats SET games_played = ?1, games_won = ?2, current_streak = ?3, max_streak = ?4, guess_distribution = ?5 WHERE id = 1",
            params![
                stats.games_played,
                stats.games_won,
                stats.current_streak,
                stats.max_streak,
                dist_json,
            ],
        )?;
        Ok(())
    }

    pub fn save_daily_result(&self, date: &str, answer: &str, won: bool, guesses: &[String]) -> SqlResult<()> {
        let guesses_json = serde_json::to_string(guesses).unwrap();
        self.conn.execute(
            "INSERT OR REPLACE INTO daily_results (date, answer, won, guesses) VALUES (?1, ?2, ?3, ?4)",
            params![date, answer, won as i32, guesses_json],
        )?;
        Ok(())
    }

    pub fn load_daily_result(&self, date: &str) -> Option<DailyResult> {
        self.conn
            .query_row(
                "SELECT date, answer, won, guesses FROM daily_results WHERE date = ?1",
                params![date],
                |row| {
                    let date: String = row.get(0)?;
                    let answer: String = row.get(1)?;
                    let won: bool = row.get::<_, i32>(2)? != 0;
                    let guesses_json: String = row.get(3)?;
                    let guesses: Vec<String> = serde_json::from_str(&guesses_json)
                        .map(|g: Vec<String>| g.iter().map(|s| normalize_persian_str(s)).collect())
                        .unwrap_or_default();
                    Ok(DailyResult { date, answer: normalize_persian_str(&answer), won, guesses })
                },
            )
            .ok()
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddWordResult {
    Added,
    Duplicate,
    Invalid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DailyResult {
    pub date: String,
    pub answer: String,
    pub won: bool,
    pub guesses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GameStats {
    pub games_played: i32,
    pub games_won: i32,
    pub current_streak: i32,
    pub max_streak: i32,
    pub guess_distribution: [i32; 6],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_word_seeding() {
        let conn = Connection::open_in_memory().unwrap();
        let mut db = DatabaseManager { conn };
        db.migrate().unwrap();
        assert_eq!(db.word_count(), 0);
        db.populate_from_default_files();
        assert_eq!(db.word_count(), 993);
        assert!(db.is_valid_word("آزادی"));
        assert!(db.is_valid_word("اسلام"));
    }
}

