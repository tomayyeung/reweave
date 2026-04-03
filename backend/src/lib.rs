use std::error::Error;
use std::fs;
use std::sync::OnceLock;

use std::collections::HashMap;

pub mod api;
mod data;

use crate::data::puzzle::Puzzle;
use crate::data::words::Trie;

static WORDS: OnceLock<Trie> = OnceLock::new();

pub fn get_words() -> &'static Trie {
    WORDS.get_or_init(|| {
        Trie::new(
            include_str!("../../wordlist/wordlist.txt")
                .lines()
                .collect(),
        )
    })
}

static PUZZLES: OnceLock<HashMap<String, Puzzle>> = OnceLock::new();

fn load_puzzles_from_dir(dir: &str) -> Result<HashMap<String, Puzzle>, Box<dyn Error>> {
    let mut puzzles = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // only process .json files
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = fs::read_to_string(&path)?;
            let puzzle: Puzzle = serde_json::from_str(&data)?;

            // use filename (without extension) as ID
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("invalid filename")?
                .to_string();

            puzzles.insert(id, puzzle);
        }
    }

    Ok(puzzles)
}

/// on dev set env variable USE_LOCAL_FILES=1 for just using a directory containing puzzles
pub fn get_puzzles() -> &'static HashMap<String, Puzzle> {
    PUZZLES.get_or_init(|| {
        if std::env::var("USE_LOCAL_FILES").is_ok() {
            load_puzzles_from_dir("../puzzles")
                .map_err(|e| format!("Error loading puzzles: {}", e))
                .unwrap()
        } else {
            // let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
            // sqlx::query_as!(Puzzle, "SELECT * FROM puzzles")
            //     .fetch_all(&pool)
            //     .await
            //     .unwrap()
            panic!("need database")
        }
    })
}
