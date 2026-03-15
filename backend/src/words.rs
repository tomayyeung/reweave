use std::collections::HashSet;

use crate::board::{Board, BoardCell};

/// For efficiency, our Trie uses a simple array to keep track of children
/// This helper function converts a byte to an index in an array of children
fn idx(b: u8) -> usize {
    assert!(b.is_ascii_lowercase(), "Invalid bit found when indexing");
    (b - b'a') as usize
}

struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    is_word: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: Default::default(),
            is_word: false,
        }
    }

    #[allow(unused)]
    fn return_words(&self) -> Vec<String> {
        let mut out = Vec::new();
        for n in 0..26_u8 {
            let Some(next_node) = &self.children[n as usize] else {
                continue;
            };
            let c = (n + b'a') as char;

            for word in next_node.return_words() {
                out.push(format!("{c}{word}"));
            }
        }

        if self.is_word {
            out.push("".to_string());
        }

        out
    }
}

/// Prefix tree - data structure used to efficiently store many words
pub struct Trie {
    root: Box<TrieNode>,
}

impl Trie {
    /// Construct a new Trie from a list of words
    pub fn new(words: Vec<&str>) -> Self {
        let mut this = Trie {
            root: Box::new(TrieNode::new()),
        };

        for word in words {
            let mut curr = &mut this.root;
            for b in word.bytes() {
                curr = curr.children[idx(b)].get_or_insert_with(|| Box::new(TrieNode::new()));
            }

            curr.is_word = true;
        }

        this
    }

    /// Search for all words in the Trie starting with prefix
    /// not necessary for this game?
    #[allow(unused)]
    fn search(&self, prefix: &str) -> Vec<String> {
        let mut curr = &self.root;
        for b in prefix.bytes() {
            let Some(node) = &curr.children[idx(b)] else {
                return vec![];
            };
            curr = node;
        }

        curr.return_words()
            .iter()
            .map(|found_word| format!("{prefix}{found_word}"))
            .collect()
    }

    /// Is this a word in the Trie?
    fn is_word(&self, word: &str) -> bool {
        let mut curr = &self.root;
        for b in word.bytes() {
            let Some(node) = &curr.children[idx(b)] else {
                return false;
            };
            curr = node;
        }

        curr.is_word
    }

    /// Are there words in the Trie starting with the given prefix?
    /// Useful for discarding unused prefixes to limit DFS
    fn is_prefix(&self, prefix: &str) -> bool {
        let mut curr = &self.root;
        for b in prefix.bytes() {
            let Some(node) = &curr.children[idx(b)] else {
                return false;
            };
            curr = node;
        }

        true
    }
}

/// Given a Board of letters and a word list, find all words
pub fn find_words(board: &Board, word_list: &Trie) -> Vec<String> {
    let mut out_hash_set = HashSet::new();

    let cells: Vec<BoardCell> = (0..board.height)
        .flat_map(|i| {
            (0..board.width)
                .map(|j| BoardCell(i, j))
                .collect::<Vec<BoardCell>>()
        })
        .collect();

    // First letter: one of the cells in the board
    for c in cells {
        out_hash_set.extend(find_words_rec(
            c,
            &mut "".to_string(),
            &mut vec![],
            board,
            word_list,
        ));
    }

    out_hash_set.into_iter().collect()
}

/// Adjacent cells
const ADJ: [(isize, isize); 8] = [
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
];

/// Recursive DFS to find all words in a board.
/// Returns a Hash Set so that words are unique.
fn find_words_rec(
    curr_cell: BoardCell,
    curr_s: &mut String,
    visited: &mut Vec<BoardCell>,
    board: &Board,
    word_list: &Trie,
) -> HashSet<String> {
    let mut out = HashSet::new();

    // Empty cell
    let Some(c) = board.get(curr_cell) else {
        return HashSet::new();
    };

    // Process current string
    if !word_list.is_prefix(curr_s) {
        return HashSet::new();
    }
    if word_list.is_word(curr_s) {
        out.insert(curr_s.clone());
    }

    // Add to curr string and visited
    curr_s.push(c);
    visited.push(curr_cell);

    // Traverse adjacent cells
    for (dx, dy) in ADJ {
        // Check for out of bounds
        if curr_cell.0 == 0 && dx == -1_isize {
            continue;
        }
        if curr_cell.0 == board.width - 1 && dx == 1 {
            continue;
        }
        if curr_cell.1 == 0 && dy == -1_isize {
            continue;
        }
        if curr_cell.1 == board.height - 1 && dy == 1 {
            continue;
        }

        let next_cell = BoardCell(
            (curr_cell.0 as isize + dx) as usize,
            (curr_cell.1 as isize + dy) as usize,
        );

        // Already visited
        if visited.contains(&next_cell) {
            continue;
        }

        out.extend(find_words_rec(next_cell, curr_s, visited, board, word_list));
    }

    // Remove from curr string and visited
    curr_s.pop();
    visited.pop();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let words = vec!["abs", "abacus", "teeth", "tusk"];

        let trie = Trie::new(words);
        assert_eq!(trie.search("a"), vec!["abacus", "abs"]);
        assert_eq!(trie.search("ab"), vec!["abacus", "abs"]);
        assert_eq!(trie.search("aba"), vec!["abacus"]);
        assert_eq!(trie.search("t"), vec!["teeth", "tusk"]);
        assert_eq!(trie.search("tu"), vec!["tusk"]);
    }

    #[test]
    fn no_search_matches() {
        let words = vec!["abs", "abacus", "teeth", "tusk"];

        let trie = Trie::new(words);
        assert_eq!(trie.search("x"), Vec::<String>::new());
        assert_eq!(trie.search("abas"), Vec::<String>::new());
    }

    #[test]
    #[should_panic]
    fn invalid_trie() {
        let words = vec!["abs", "abacus", "teeth", "tusk", "你好"];

        Trie::new(words);
    }

    #[test]
    #[should_panic]
    fn invalid_search() {
        let words = vec!["hello", "hey", "hi"];

        let trie = Trie::new(words);
        trie.search("H");
    }

    #[test]
    fn is_valid_prefix() {
        let words = vec!["test", "teach", "toaster"];

        let trie = Trie::new(words);
        assert!(trie.is_prefix("te"));
    }

    #[test]
    fn is_not_valid_prefix() {
        let words = vec!["test", "teach", "toaster"];

        let trie = Trie::new(words);
        assert!(!trie.is_prefix("ta"));
    }

    #[test]
    fn find1() {
        let full_word_list = Trie::new(vec!["abc", "dab", "cab", "daba", "abe"]);
        let board = Board::create(2, 2, vec!['a', 'b', 'c', 'd']);

        let mut found_words = find_words(&board, &full_word_list);
        found_words.sort();

        assert_eq!(found_words, vec!["abc", "cab", "dab"]);
    }

    #[test]
    fn find2() {
        let full_word_list = Trie::new(vec!["both", "broth", "foul", "trouble", "blur"]);
        let board = Board::create(3, 3, vec!['t', 'r', 'b', 'h', 'o', 'u', 'f', 'l', 'y']);

        let mut found_words = find_words(&board, &full_word_list);
        found_words.sort();

        assert_eq!(found_words, vec!["both", "broth", "foul"]);
    }

    #[test]
    fn find3() {
        let full_word_list = Trie::new(vec!["both", "broth", "foul", "trouble", "blur"]);
        let board = Board::create(3, 3, vec!['t', 'r', 'b', 'h', 'o', 'u', 'f', 'l', ' ']);

        let mut found_words = find_words(&board, &full_word_list);
        found_words.sort();

        assert_eq!(found_words, vec!["both", "broth", "foul"]);
    }
}
