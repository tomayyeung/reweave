use std::collections::HashSet;
use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::board::*;
use crate::words::*;

/// A struct for the output of comparing words in a board
/// to words a puzzle requires.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Words {
    pub found: Vec<String>,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

/// A list of words that the player uses to create
/// a board.
#[derive(Default, Serialize)]
pub struct Puzzle {
    pub width: usize,
    pub height: usize,
    /// Empty squares in the puzzle that the player knows will be empty
    pub holes: Vec<BoardCell>,
    /// Filled squares in the puzzle that the player is given at the start
    pub starting_letters: Vec<(BoardCell, char)>,
    pub words: HashSet<String>,
}

impl Puzzle {
    #[allow(unused)]
    pub fn from_board(board: &Board, word_list: &Trie) -> Self {
        Puzzle {
            width: board.width,
            height: board.height,
            holes: board.get_empty_cells(),
            starting_letters: vec![],
            words: find_words(board, word_list).into_iter().collect(),
        }
    }

    pub fn create(
        width: usize,
        height: usize,
        chars: Vec<char>,
        words: HashSet<String>,
    ) -> Result<Self, &'static str> {
        if width * height != chars.len() {
            return Err("Width and height do not match length of chars");
        }

        let mut holes = Vec::new();
        let mut starting_letters = Vec::new();

        let mut i = 0;
        for x in 0..height {
            for y in 0..width {
                let c = *chars.get(i).unwrap();

                if c == '#' {
                    holes.push(BoardCell(x, y));
                } else if c.is_ascii_lowercase() {
                    starting_letters.push((BoardCell(x, y), c));
                }

                i += 1;
            }
        }

        Ok(Puzzle {
            width,
            height,
            holes,
            starting_letters,
            words,
        })
    }

    pub fn compare_found_words(&self, found_words: Vec<String>) -> Words {
        let found_words_set: HashSet<_> = found_words.into_iter().collect();

        Words {
            found: found_words_set.intersection(&self.words).cloned().collect(),
            missing: self.words.difference(&found_words_set).cloned().collect(),
            extra: found_words_set.difference(&self.words).cloned().collect(),
        }
    }

    pub fn to_file(&self, path: &str) {
        let file = File::create(path).unwrap();
        serde_json::to_writer(file, &self).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn cmp_words() {
        let puzzle = Puzzle {
            words: HashSet::from(["abc".to_string(), "def".to_string()]),
            ..Default::default()
        };

        assert_eq!(
            puzzle.compare_found_words(vec!["abc".to_string(), "ghi".to_string()]),
            Words {
                found: vec!["abc".to_string()],
                missing: vec!["def".to_string()],
                extra: vec!["ghi".to_string()],
            }
        );
    }

    #[test]
    fn cmp_puzzle_from() {
        let board = Board::create(2, 2, vec!['c', 'a', 't', 's']);
        let word_list = Trie::new(vec!["act", "cat", "cats"]);
        let puzzle = Puzzle::from_board(&board, &word_list);

        let mut words = puzzle.compare_found_words(vec![
            "act".to_string(),
            "cat".to_string(),
            "cart".to_string(),
        ]);
        words.found.sort();

        assert_eq!(
            words,
            Words {
                found: vec!["act".to_string(), "cat".to_string()],
                missing: vec!["cats".to_string()],
                extra: vec!["cart".to_string()],
            }
        );
    }
}
