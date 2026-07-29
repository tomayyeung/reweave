//! Recomputes persisted puzzle word lists from their solved boards.
//!
//! Run this after changing `wordlist/wordlist.txt` so every existing puzzle's
//! `words` column matches the current dictionary while preserving its `answer`.

use std::error::Error;

use reweave::common::{board, words};
use reweave::db;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct PuzzleAnswerRow {
    id: Uuid,
    width: i32,
    height: i32,
    answer: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let word_list = words::Trie::new(
        include_str!("../../../wordlist/wordlist.txt")
            .lines()
            .collect(),
    );

    let rows = sqlx::query_as::<_, PuzzleAnswerRow>(
        "SELECT id, width, height, answer FROM puzzles ORDER BY created_at ASC",
    )
    .fetch_all(db::get_puzzles_pool())
    .await?;
    let puzzle_count = rows.len();
    let mut transaction = db::get_puzzles_pool().begin().await?;

    // RLS permits puzzle updates for admins. Maintenance scripts run outside a
    // user request, so set the transaction-local role explicitly.
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind("app.role")
        .bind("admin")
        .execute(&mut *transaction)
        .await?;

    for row in rows {
        let board = board::Board::create(
            row.width as usize,
            row.height as usize,
            row.answer.chars().collect(),
        )
        .map_err(|error| format!("invalid answer for puzzle {}: {error}", row.id))?;
        let mut found_words = board::find_words(&board, &word_list);
        found_words.sort();

        let result = sqlx::query("UPDATE puzzles SET words = $1 WHERE id = $2")
            .bind(&found_words as &[String])
            .bind(row.id)
            .execute(&mut *transaction)
            .await?;

        if result.rows_affected() != 1 {
            return Err(format!(
                "expected to update puzzle {}, updated {} rows",
                row.id,
                result.rows_affected()
            )
            .into());
        }

        println!("updated puzzle {} with {} words", row.id, found_words.len());
    }

    transaction.commit().await?;

    println!("updated {puzzle_count} puzzles");

    Ok(())
}
