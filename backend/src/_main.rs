// use std::collections::HashMap;
// use std::error::Error;
// use std::fs;
// use std::sync::Arc;

// use reweave::{api, puzzle, words};

// const WORDS: &str = include_str!("../../wordlist/wordlist.txt");

// fn load_puzzles(dir: &str) -> Result<HashMap<String, puzzle::Puzzle>, Box<dyn Error>> {
//     let mut puzzles = HashMap::new();

//     for entry in fs::read_dir(dir)? {
//         let entry = entry?;
//         let path = entry.path();

//         // only process .json files
//         if path.extension().and_then(|s| s.to_str()) == Some("json") {
//             let data = fs::read_to_string(&path)?;
//             let puzzle: puzzle::Puzzle = serde_json::from_str(&data)?;

//             // use filename (without extension) as ID
//             let id = path
//                 .file_stem()
//                 .and_then(|s| s.to_str())
//                 .ok_or("invalid filename")?
//                 .to_string();

//             puzzles.insert(id, puzzle);
//         }
//     }

//     Ok(puzzles)
// }

// #[tokio::main]
// async fn main() {
//     // Initialize words
//     let full_word_list = Arc::new(words::Trie::new(WORDS.split("\n").collect()));

//     // Initialize puzzles
//     let puzzles_path = format!("{}/../puzzles", env!("CARGO_MANIFEST_DIR"));
//     let all_puzzles = Arc::new(load_puzzles(&puzzles_path).expect("failed to load puzzles"));

//     // Initialize APIs
//     let app = api::router(full_word_list, all_puzzles, Arc::new(puzzles_path));

//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

//     axum::serve(listener, app).await.unwrap();
// }
