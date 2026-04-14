use sqlx::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::OnceLock;
use tokio::sync::OnceCell;

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

pub static PUZZLES_POOL: OnceLock<PgPool> = OnceLock::new();
static PUZZLES: OnceCell<HashMap<String, Puzzle>> = OnceCell::const_new();

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

#[derive(sqlx::FromRow)]
struct PuzzleRow {
    pub id: String,
    pub width: i32,
    pub height: i32,
    pub letters: String,
    pub words: Vec<String>,
}

impl From<PuzzleRow> for Puzzle {
    fn from(row: PuzzleRow) -> Self {
        Puzzle {
            width: row.width as usize,
            height: row.height as usize,
            letters: row.letters,
            words: row.words.into_iter().collect(),
        }
    }
}

pub fn get_puzzles_pool() -> &'static PgPool {
    PUZZLES_POOL
        .get_or_init(|| PgPool::connect_lazy(&std::env::var("DATABASE_URL").unwrap()).unwrap())
}

/// Loads in all puzzles stored in the database
///
/// caveat: stale data. if a new puzzle is created, it won't appear. Need to reload on write or just query db directly
async fn load_puzzles_from_db() -> Result<HashMap<String, Puzzle>, Box<dyn Error>> {
    let pool = get_puzzles_pool();
    let rows = sqlx::query_as!(PuzzleRow, "SELECT * FROM puzzles")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|p| (p.id.clone(), Puzzle::from(p)))
        .collect())
}

/// on dev set env variable USE_LOCAL_FILES=1 for just using a directory containing puzzles
pub async fn get_puzzles() -> &'static HashMap<String, Puzzle> {
    PUZZLES
        .get_or_init(|| async {
            if std::env::var("USE_LOCAL_FILES").is_ok() {
                load_puzzles_from_dir("../puzzles")
            } else {
                load_puzzles_from_db().await
            }
            .map_err(|e| format!("Error loading puzzles: {}", e))
            .unwrap()
        })
        .await
}
