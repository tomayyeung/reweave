use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

const BLANK: char = '_';
const HOLE: char = '!';
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Cell(usize, usize);

#[derive(Debug, Deserialize)]
struct Puzzle {
    name: String,
    width: usize,
    height: usize,
    letters: String,
    words: HashSet<String>,
}

#[derive(Clone)]
struct Board {
    width: usize,
    height: usize,
    cells: Vec<Option<char>>,
    holes: Vec<bool>,
}

impl Board {
    fn from_puzzle(puzzle: &Puzzle) -> Result<Self, String> {
        if puzzle.width * puzzle.height != puzzle.letters.chars().count() {
            return Err("width and height do not match letters length".to_string());
        }

        let mut cells = Vec::new();
        let mut holes = Vec::new();
        for c in puzzle.letters.chars() {
            match c {
                BLANK => {
                    cells.push(None);
                    holes.push(false);
                }
                HOLE => {
                    cells.push(None);
                    holes.push(true);
                }
                c if c.is_ascii_lowercase() => {
                    cells.push(Some(c));
                    holes.push(false);
                }
                _ => return Err(format!("invalid board character: {c}")),
            }
        }

        Ok(Self {
            width: puzzle.width,
            height: puzzle.height,
            cells,
            holes,
        })
    }

    fn idx(&self, cell: Cell) -> usize {
        cell.0 * self.width + cell.1
    }

    fn cell(&self, idx: usize) -> Cell {
        Cell(idx / self.width, idx % self.width)
    }

    fn set(&mut self, idx: usize, c: char) {
        self.cells[idx] = Some(c);
    }

    fn clear(&mut self, idx: usize) {
        self.cells[idx] = None;
    }

    fn is_hole_idx(&self, idx: usize) -> bool {
        self.holes[idx]
    }

    fn is_open_idx(&self, idx: usize) -> bool {
        !self.holes[idx]
    }

    fn is_complete(&self) -> bool {
        self.cells
            .iter()
            .enumerate()
            .all(|(idx, c)| self.is_hole_idx(idx) || c.is_some())
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for row in 0..self.height {
            if row > 0 {
                out.push('\n');
            }
            for col in 0..self.width {
                let idx = self.idx(Cell(row, col));
                let c = if self.is_hole_idx(idx) {
                    HOLE
                } else {
                    self.cells[idx].unwrap_or(BLANK)
                };
                out.push(c);
                if col + 1 < self.width {
                    out.push(' ');
                }
            }
        }
        out
    }
}

#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    is_word: bool,
}

struct Trie {
    root: Box<TrieNode>,
}

impl Trie {
    fn new(words: impl IntoIterator<Item = String>) -> Self {
        let mut trie = Trie {
            root: Box::new(TrieNode::default()),
        };

        for word in words {
            if word.is_empty() || !word.bytes().all(|b| b.is_ascii_lowercase()) {
                continue;
            }
            let mut curr = &mut trie.root;
            for b in word.bytes() {
                curr = curr.children[(b - b'a') as usize].get_or_insert_with(Default::default);
            }
            curr.is_word = true;
        }

        trie
    }

    fn step<'a>(&'a self, node: &'a TrieNode, c: char) -> Option<&'a TrieNode> {
        if !c.is_ascii_lowercase() {
            return None;
        }
        node.children[(c as u8 - b'a') as usize].as_deref()
    }
}

#[derive(Clone)]
struct WordPaths {
    word: String,
    paths: Vec<Vec<usize>>,
}

struct Solver {
    target_words: HashSet<String>,
    full_words: Trie,
    word_paths: Vec<WordPaths>,
    rare_rank: [usize; 26],
}

#[derive(Debug, Eq, PartialEq)]
enum SearchOutcome {
    Found,
    NotFound,
    NodeLimit,
}

#[derive(Default)]
struct SearchStats {
    nodes: u64,
    word_nodes: u64,
    cell_nodes: u64,
    path_attempts: u64,
    memo_hits: u64,
    target_prunes: u64,
    extra_prunes: u64,
    dead_ends: u64,
    complete_boards: u64,
}

#[derive(Clone, Copy)]
struct SolverOptions {
    disable_memo: bool,
    full_extra_checks: bool,
    cell_search: bool,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            disable_memo: true,
            full_extra_checks: false,
            cell_search: false,
        }
    }
}

impl Solver {
    fn new(board: &Board, target_words: HashSet<String>, full_words: Trie) -> Result<Self, String> {
        let mut words: Vec<String> = target_words.iter().cloned().collect();
        words.sort_by_key(|w| (word_rarity_key(w, &target_words), w.len(), w.clone()));

        let mut word_paths = Vec::new();
        for word in words {
            let paths = precompute_paths(board, &word);
            if paths.is_empty() {
                return Err(format!("target word has no realizable path: {word}"));
            }
            word_paths.push(WordPaths { word, paths });
        }

        Ok(Self {
            rare_rank: rare_rank(&target_words),
            target_words,
            full_words,
            word_paths,
        })
    }

    #[allow(dead_code)]
    fn solve(&self, board: &mut Board) -> bool {
        let mut stats = SearchStats::default();
        self.solve_with_stats(board, &mut stats, None, SolverOptions::default())
            == SearchOutcome::Found
    }

    fn solve_with_stats(
        &self,
        board: &mut Board,
        stats: &mut SearchStats,
        max_nodes: Option<u64>,
        options: SolverOptions,
    ) -> SearchOutcome {
        if self.has_extra_complete_word(board) {
            stats.extra_prunes += 1;
            return SearchOutcome::NotFound;
        }

        if options.cell_search {
            return self.solve_inner(
                board,
                stats,
                max_nodes,
                &mut HashSet::new(),
                &vec![false; self.word_paths.len()],
                options,
            );
        }

        self.solve_by_words_inner(
            board,
            &mut vec![false; self.word_paths.len()],
            stats,
            max_nodes,
            &mut HashSet::new(),
            options,
        )
    }

    fn solve_by_words_inner(
        &self,
        board: &mut Board,
        placed_words: &mut [bool],
        stats: &mut SearchStats,
        max_nodes: Option<u64>,
        failed_states: &mut HashSet<String>,
        options: SolverOptions,
    ) -> SearchOutcome {
        stats.nodes += 1;
        stats.word_nodes += 1;
        if max_nodes.is_some_and(|max_nodes| stats.nodes > max_nodes) {
            return SearchOutcome::NodeLimit;
        }

        let state_key = state_key(board, placed_words);
        if !options.disable_memo && failed_states.contains(&state_key) {
            stats.memo_hits += 1;
            return SearchOutcome::NotFound;
        }

        if options.full_extra_checks && self.has_extra_complete_word(board) {
            stats.extra_prunes += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        if !self.target_feasible(board) {
            stats.target_prunes += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        let Some((word_idx, mut paths)) = self.next_word_and_paths(board, placed_words) else {
            let outcome = self.solve_inner(
                board,
                stats,
                max_nodes,
                failed_states,
                placed_words,
                options,
            );
            if outcome == SearchOutcome::NotFound && !options.disable_memo {
                failed_states.insert(state_key);
            }
            return outcome;
        };

        paths.sort_by_key(|path| self.path_order_key(board, &self.word_paths[word_idx].word, path));
        if paths.is_empty() {
            stats.dead_ends += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        placed_words[word_idx] = true;
        for path in paths {
            stats.path_attempts += 1;
            let Some(changed) = assign_path(board, &self.word_paths[word_idx].word, &path) else {
                continue;
            };

            let outcome = if self.extra_prune_after_assignment(board, &changed, options) {
                stats.extra_prunes += 1;
                SearchOutcome::NotFound
            } else {
                self.solve_by_words_inner(
                    board,
                    placed_words,
                    stats,
                    max_nodes,
                    failed_states,
                    options,
                )
            };
            if outcome != SearchOutcome::Found {
                unassign_path(board, &changed);
            }

            match outcome {
                SearchOutcome::Found => return SearchOutcome::Found,
                SearchOutcome::NodeLimit => {
                    placed_words[word_idx] = false;
                    return SearchOutcome::NodeLimit;
                }
                SearchOutcome::NotFound => {}
            }
        }
        placed_words[word_idx] = false;

        if !options.disable_memo {
            failed_states.insert(state_key);
        }
        SearchOutcome::NotFound
    }

    fn solve_inner(
        &self,
        board: &mut Board,
        stats: &mut SearchStats,
        max_nodes: Option<u64>,
        failed_states: &mut HashSet<String>,
        placed_words: &[bool],
        options: SolverOptions,
    ) -> SearchOutcome {
        stats.nodes += 1;
        stats.cell_nodes += 1;
        if max_nodes.is_some_and(|max_nodes| stats.nodes > max_nodes) {
            return SearchOutcome::NodeLimit;
        }

        let state_key = state_key(board, placed_words);
        if !options.disable_memo && failed_states.contains(&state_key) {
            stats.memo_hits += 1;
            return SearchOutcome::NotFound;
        }

        if options.full_extra_checks && self.has_extra_complete_word(board) {
            stats.extra_prunes += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        if !self.target_feasible(board) {
            stats.target_prunes += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        if board.is_complete() {
            stats.complete_boards += 1;
            return if self.found_words(board) == self.target_words {
                SearchOutcome::Found
            } else {
                SearchOutcome::NotFound
            };
        }

        let Some((idx, mut candidates)) = self.next_cell_and_candidates(board) else {
            stats.dead_ends += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        };

        candidates.sort_by_key(|c| self.rare_rank[(*c as u8 - b'a') as usize]);
        if candidates.is_empty() {
            stats.dead_ends += 1;
            if !options.disable_memo {
                failed_states.insert(state_key);
            }
            return SearchOutcome::NotFound;
        }

        for c in candidates {
            board.set(idx, c);
            let outcome = if self.extra_prune_after_assignment(board, &[idx], options) {
                stats.extra_prunes += 1;
                SearchOutcome::NotFound
            } else {
                self.solve_inner(
                    board,
                    stats,
                    max_nodes,
                    failed_states,
                    placed_words,
                    options,
                )
            };
            if outcome != SearchOutcome::Found {
                board.clear(idx);
            }
            match outcome {
                SearchOutcome::Found => return SearchOutcome::Found,
                SearchOutcome::NodeLimit => return SearchOutcome::NodeLimit,
                SearchOutcome::NotFound => {}
            }
        }

        if !options.disable_memo {
            failed_states.insert(state_key);
        }
        SearchOutcome::NotFound
    }

    fn next_word_and_paths(
        &self,
        board: &Board,
        placed_words: &[bool],
    ) -> Option<(usize, Vec<Vec<usize>>)> {
        let mut best: Option<(usize, Vec<Vec<usize>>)> = None;

        for (idx, word_paths) in self.word_paths.iter().enumerate() {
            if placed_words[idx] {
                continue;
            }

            let paths: Vec<Vec<usize>> = word_paths
                .paths
                .iter()
                .filter(|path| path_compatible(board, &word_paths.word, path))
                .cloned()
                .collect();

            match &best {
                None => best = Some((idx, paths)),
                Some((best_idx, best_paths)) => {
                    let best_word = &self.word_paths[*best_idx].word;
                    if paths.len() < best_paths.len()
                        || (paths.len() == best_paths.len()
                            && word_paths.word.len() > best_word.len())
                        || (paths.len() == best_paths.len()
                            && word_paths.word.len() == best_word.len()
                            && word_rarity_key(&word_paths.word, &self.target_words)
                                < word_rarity_key(best_word, &self.target_words))
                    {
                        best = Some((idx, paths));
                    }
                }
            }
        }

        best
    }

    fn path_order_key(&self, board: &Board, word: &str, path: &[usize]) -> (usize, usize, usize) {
        let new_assignments = path
            .iter()
            .filter(|idx| board.cells[**idx].is_none())
            .count();
        let rarity = word
            .bytes()
            .zip(path)
            .map(|(b, idx)| self.rare_rank[(b - b'a') as usize] * 100 + idx)
            .min()
            .unwrap_or(usize::MAX);

        (usize::MAX - new_assignments, rarity, path[0])
    }

    fn next_cell_and_candidates(&self, board: &Board) -> Option<(usize, Vec<char>)> {
        let mut best: Option<(usize, Vec<char>, usize)> = None;

        for idx in 0..board.cells.len() {
            if board.is_hole_idx(idx) || board.cells[idx].is_some() {
                continue;
            }

            let candidates = self.candidates_for_cell(board, idx);
            let constraint_count = self.constraint_count(idx);
            match &best {
                None => best = Some((idx, candidates, constraint_count)),
                Some((_, best_candidates, best_constraints)) => {
                    if candidates.len() < best_candidates.len()
                        || (candidates.len() == best_candidates.len()
                            && constraint_count > *best_constraints)
                    {
                        best = Some((idx, candidates, constraint_count));
                    }
                }
            }
        }

        best.map(|(idx, candidates, _)| (idx, candidates))
    }

    fn candidates_for_cell(&self, board: &Board, idx: usize) -> Vec<char> {
        let mut out = HashSet::new();
        for word_paths in &self.word_paths {
            for path in &word_paths.paths {
                if !path.contains(&idx) || !path_compatible(board, &word_paths.word, path) {
                    continue;
                }
                for (pos, path_idx) in path.iter().enumerate() {
                    if *path_idx == idx {
                        out.insert(word_paths.word.as_bytes()[pos] as char);
                    }
                }
            }
        }

        if out.is_empty() {
            ('a'..='z').collect()
        } else {
            out.into_iter().collect()
        }
    }

    fn constraint_count(&self, idx: usize) -> usize {
        self.word_paths
            .iter()
            .map(|word_paths| {
                word_paths
                    .paths
                    .iter()
                    .filter(|path| path.contains(&idx))
                    .count()
            })
            .sum()
    }

    fn target_feasible(&self, board: &Board) -> bool {
        self.word_paths.iter().all(|word_paths| {
            word_paths
                .paths
                .iter()
                .any(|path| path_compatible(board, &word_paths.word, path))
        })
    }

    fn has_extra_complete_word(&self, board: &Board) -> bool {
        self.collect_complete_words(board, true)
            .into_iter()
            .any(|word| !self.target_words.contains(&word))
    }

    fn has_extra_complete_word_from(&self, board: &Board, changed: &[usize]) -> bool {
        if changed.is_empty() {
            return false;
        }

        self.collect_complete_words_from(board, changed)
            .into_iter()
            .any(|word| !self.target_words.contains(&word))
    }

    fn extra_prune_after_assignment(
        &self,
        board: &Board,
        changed: &[usize],
        options: SolverOptions,
    ) -> bool {
        if options.full_extra_checks {
            self.has_extra_complete_word(board)
        } else {
            self.has_extra_complete_word_from(board, changed)
        }
    }

    fn found_words(&self, board: &Board) -> HashSet<String> {
        self.collect_complete_words(board, false)
    }

    fn print_static_stats(&self, board: &Board) {
        let holes = board.holes.iter().filter(|is_hole| **is_hole).count();
        let fixed = board.cells.iter().filter(|cell| cell.is_some()).count();
        let blanks = board.cells.len() - holes - fixed;

        println!("board: {}x{}", board.width, board.height);
        println!("fillable cells: {}", board.cells.len() - holes);
        println!("fixed cells: {fixed}");
        println!("blank cells: {blanks}");
        println!("holes: {holes}");
        println!("target words: {}", self.word_paths.len());

        let mut path_counts: Vec<_> = self
            .word_paths
            .iter()
            .map(|word_paths| (word_paths.word.as_str(), word_paths.paths.len()))
            .collect();
        path_counts.sort_by_key(|(_, count)| *count);

        let total_paths: usize = path_counts.iter().map(|(_, count)| *count).sum();
        let min_paths = path_counts.first().map(|(_, count)| *count).unwrap_or(0);
        let max_paths = path_counts.last().map(|(_, count)| *count).unwrap_or(0);
        println!("target path counts: total={total_paths}, min={min_paths}, max={max_paths}");

        println!("tightest target path counts:");
        for (word, count) in path_counts.iter().take(10) {
            println!("  {word}: {count}");
        }

        println!("loosest target path counts:");
        for (word, count) in path_counts.iter().rev().take(10) {
            println!("  {word}: {count}");
        }

        if let Some((idx, mut candidates)) = self.next_cell_and_candidates(board) {
            candidates.sort_by_key(|c| self.rare_rank[(*c as u8 - b'a') as usize]);
            let Cell(row, col) = board.cell(idx);
            println!(
                "initial MRV cell: ({row}, {col}), candidates={} ({})",
                candidates.len(),
                candidates.iter().collect::<String>()
            );
        }
    }

    fn collect_complete_words(&self, board: &Board, stop_on_extra: bool) -> HashSet<String> {
        let mut out = HashSet::new();
        let mut visited = vec![false; board.cells.len()];
        let mut curr = String::new();

        for idx in 0..board.cells.len() {
            if board.is_hole_idx(idx) || board.cells[idx].is_none() {
                continue;
            }
            self.collect_complete_words_rec(
                board,
                idx,
                &self.full_words.root,
                &mut visited,
                &mut curr,
                &mut out,
                stop_on_extra,
            );
            if stop_on_extra && out.iter().any(|word| !self.target_words.contains(word)) {
                return out;
            }
        }

        out
    }

    fn collect_complete_words_from(&self, board: &Board, changed: &[usize]) -> HashSet<String> {
        let mut out = HashSet::new();
        let changed: HashSet<usize> = changed.iter().copied().collect();

        for start in 0..board.cells.len() {
            if board.is_hole_idx(start) || board.cells[start].is_none() {
                continue;
            }

            let mut visited = vec![false; board.cells.len()];
            let mut curr = String::new();
            self.collect_complete_words_from_rec(
                board,
                start,
                &self.full_words.root,
                &mut visited,
                &mut curr,
                &mut out,
                &changed,
                false,
            );
        }

        out
    }

    fn collect_complete_words_from_rec(
        &self,
        board: &Board,
        idx: usize,
        node: &TrieNode,
        visited: &mut [bool],
        curr: &mut String,
        out: &mut HashSet<String>,
        changed: &HashSet<usize>,
        touched_changed: bool,
    ) {
        let Some(c) = board.cells[idx] else {
            return;
        };
        let Some(next_node) = self.full_words.step(node, c) else {
            return;
        };

        visited[idx] = true;
        curr.push(c);
        let touched_changed = touched_changed || changed.contains(&idx);

        if touched_changed && next_node.is_word {
            out.insert(curr.clone());
            if !self.target_words.contains(curr) {
                curr.pop();
                visited[idx] = false;
                return;
            }
        }

        for next in neighbors(board, idx) {
            if !visited[next] && board.cells[next].is_some() {
                self.collect_complete_words_from_rec(
                    board,
                    next,
                    next_node,
                    visited,
                    curr,
                    out,
                    changed,
                    touched_changed,
                );
                if out.iter().any(|word| !self.target_words.contains(word)) {
                    break;
                }
            }
        }

        curr.pop();
        visited[idx] = false;
    }

    fn collect_complete_words_rec(
        &self,
        board: &Board,
        idx: usize,
        node: &TrieNode,
        visited: &mut [bool],
        curr: &mut String,
        out: &mut HashSet<String>,
        stop_on_extra: bool,
    ) {
        let Some(c) = board.cells[idx] else {
            return;
        };
        let Some(next_node) = self.full_words.step(node, c) else {
            return;
        };

        visited[idx] = true;
        curr.push(c);

        if next_node.is_word {
            out.insert(curr.clone());
            if stop_on_extra && !self.target_words.contains(curr) {
                curr.pop();
                visited[idx] = false;
                return;
            }
        }

        for next in neighbors(board, idx) {
            if !visited[next] && board.cells[next].is_some() {
                self.collect_complete_words_rec(
                    board,
                    next,
                    next_node,
                    visited,
                    curr,
                    out,
                    stop_on_extra,
                );
                if stop_on_extra && out.iter().any(|word| !self.target_words.contains(word)) {
                    break;
                }
            }
        }

        curr.pop();
        visited[idx] = false;
    }
}

fn precompute_paths(board: &Board, word: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let bytes = word.as_bytes();
    let mut visited = vec![false; board.cells.len()];
    let mut path = Vec::new();

    for idx in 0..board.cells.len() {
        precompute_paths_rec(board, bytes, idx, 0, &mut visited, &mut path, &mut out);
    }

    out
}

fn precompute_paths_rec(
    board: &Board,
    word: &[u8],
    idx: usize,
    pos: usize,
    visited: &mut [bool],
    path: &mut Vec<usize>,
    out: &mut Vec<Vec<usize>>,
) {
    if board.is_hole_idx(idx) || visited[idx] {
        return;
    }

    let want = word[pos] as char;
    if let Some(have) = board.cells[idx] {
        if have != want {
            return;
        }
    }

    visited[idx] = true;
    path.push(idx);

    if pos + 1 == word.len() {
        out.push(path.clone());
    } else {
        for next in neighbors(board, idx) {
            precompute_paths_rec(board, word, next, pos + 1, visited, path, out);
        }
    }

    path.pop();
    visited[idx] = false;
}

fn path_compatible(board: &Board, word: &str, path: &[usize]) -> bool {
    path.iter().enumerate().all(|(pos, idx)| {
        !board.is_hole_idx(*idx)
            && board.cells[*idx].is_none_or(|c| c == word.as_bytes()[pos] as char)
    })
}

fn assign_path(board: &mut Board, word: &str, path: &[usize]) -> Option<Vec<usize>> {
    let mut changed = Vec::new();

    for (pos, idx) in path.iter().enumerate() {
        let c = word.as_bytes()[pos] as char;
        match board.cells[*idx] {
            Some(existing) if existing != c => {
                unassign_path(board, &changed);
                return None;
            }
            Some(_) => {}
            None => {
                board.set(*idx, c);
                changed.push(*idx);
            }
        }
    }

    Some(changed)
}

fn unassign_path(board: &mut Board, changed: &[usize]) {
    for idx in changed.iter().rev() {
        board.clear(*idx);
    }
}

fn state_key(board: &Board, placed_words: &[bool]) -> String {
    let mut key = String::with_capacity(board.cells.len() + placed_words.len() + 1);
    for idx in 0..board.cells.len() {
        if board.is_hole_idx(idx) {
            key.push(HOLE);
        } else {
            key.push(board.cells[idx].unwrap_or(BLANK));
        }
    }
    key.push('|');
    for placed in placed_words {
        key.push(if *placed { '1' } else { '0' });
    }
    key
}

fn neighbors(board: &Board, idx: usize) -> Vec<usize> {
    let Cell(row, col) = board.cell(idx);
    let mut out = Vec::new();
    for (dr, dc) in ADJ {
        let next_row = row as isize + dr;
        let next_col = col as isize + dc;
        if next_row < 0
            || next_col < 0
            || next_row >= board.height as isize
            || next_col >= board.width as isize
        {
            continue;
        }
        let next = board.idx(Cell(next_row as usize, next_col as usize));
        if board.is_open_idx(next) {
            out.push(next);
        }
    }
    out
}

fn rare_rank(words: &HashSet<String>) -> [usize; 26] {
    let mut counts = [0usize; 26];
    for word in words {
        for b in word.bytes() {
            if b.is_ascii_lowercase() {
                counts[(b - b'a') as usize] += 1;
            }
        }
    }

    let mut letters: Vec<usize> = (0..26).collect();
    letters.sort_by_key(|idx| (counts[*idx], *idx));

    let mut rank = [0usize; 26];
    for (pos, idx) in letters.into_iter().enumerate() {
        rank[idx] = pos;
    }
    rank
}

fn word_rarity_key(word: &str, words: &HashSet<String>) -> usize {
    let rank = rare_rank(words);
    word.bytes()
        .filter(|b| b.is_ascii_lowercase())
        .map(|b| rank[(b - b'a') as usize])
        .min()
        .unwrap_or(usize::MAX)
}

fn load_words(path: &Path) -> Result<Vec<String>, String> {
    let contents =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect())
}

struct Args {
    puzzle: PathBuf,
    dict: PathBuf,
    stats: bool,
    max_nodes: Option<u64>,
    options: SolverOptions,
}

fn parse_args() -> Result<Args, String> {
    let mut args = env::args().skip(1);
    let mut dict = PathBuf::from("wordlist/wordlist.txt");
    let mut puzzle = None;
    let mut stats = false;
    let mut max_nodes = None;
    let mut options = SolverOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dict" => {
                let Some(path) = args.next() else {
                    return Err("--dict requires a path".to_string());
                };
                dict = PathBuf::from(path);
            }
            "--stats" => stats = true,
            "--enable-memo" => options.disable_memo = false,
            "--disable-memo" => options.disable_memo = true,
            "--full-extra-checks" => options.full_extra_checks = true,
            "--cell-search" => options.cell_search = true,
            "--max-nodes" => {
                let Some(value) = args.next() else {
                    return Err("--max-nodes requires a number".to_string());
                };
                max_nodes = Some(
                    value
                        .parse()
                        .map_err(|_| "--max-nodes requires a number".to_string())?,
                );
            }
            _ if puzzle.is_none() => puzzle = Some(PathBuf::from(arg)),
            _ => return Err(usage()),
        }
    }

    let Some(puzzle) = puzzle else {
        return Err(usage());
    };

    Ok(Args {
        puzzle,
        dict,
        stats,
        max_nodes,
        options,
    })
}

fn usage() -> String {
    "usage: solver [--stats] [--max-nodes N] [--cell-search] [--enable-memo] [--disable-memo] [--full-extra-checks] [--dict path/to/wordlist.txt] path/to/puzzle.json|-"
        .to_string()
}

fn read_puzzle_json(path: &Path) -> Result<String, String> {
    if path == Path::new("-") {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        Ok(input)
    } else {
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let puzzle_json = read_puzzle_json(&args.puzzle)?;
    let puzzle: Puzzle = serde_json::from_str(&puzzle_json)
        .map_err(|e| format!("failed to parse puzzle json: {e}"))?;
    let mut board = Board::from_puzzle(&puzzle)?;
    let dict_words = load_words(&args.dict)?;
    let full_words = Trie::new(dict_words);
    let solver = Solver::new(&board, puzzle.words, full_words)?;

    if args.stats {
        solver.print_static_stats(&board);
        let mut stats = SearchStats::default();
        let started = Instant::now();
        let outcome = solver.solve_with_stats(&mut board, &mut stats, args.max_nodes, args.options);
        let elapsed = started.elapsed();
        let nodes_per_sec = stats.nodes as f64 / elapsed.as_secs_f64().max(0.001);
        println!(
            "search mode: {}",
            if args.options.cell_search {
                "cell MRV only"
            } else {
                "word paths + cell MRV fallback"
            }
        );
        println!(
            "memoization: {}",
            if args.options.disable_memo {
                "off"
            } else {
                "on"
            }
        );
        println!(
            "extra checks: {}",
            if args.options.full_extra_checks {
                "full board"
            } else {
                "changed cells"
            }
        );
        println!("search outcome: {outcome:?}");
        println!("elapsed: {:.3}s", elapsed.as_secs_f64());
        println!("nodes: {}", stats.nodes);
        println!("word nodes: {}", stats.word_nodes);
        println!("cell fallback nodes: {}", stats.cell_nodes);
        println!("path attempts: {}", stats.path_attempts);
        println!("memo hits: {}", stats.memo_hits);
        println!("nodes/sec: {:.0}", nodes_per_sec);
        println!("target feasibility prunes: {}", stats.target_prunes);
        println!("extra word prunes: {}", stats.extra_prunes);
        println!("dead ends: {}", stats.dead_ends);
        println!("complete boards checked: {}", stats.complete_boards);
        if outcome == SearchOutcome::Found {
            println!("Solved {}", puzzle.name);
            println!("{}", board.render());
            return Ok(());
        }
        if outcome == SearchOutcome::NodeLimit {
            return Ok(());
        }
        return Err("no solution found".to_string());
    }

    match solver.solve_with_stats(
        &mut board,
        &mut SearchStats::default(),
        args.max_nodes,
        args.options,
    ) {
        SearchOutcome::Found => {
            println!("Solved {}", puzzle.name);
            println!("{}", board.render());
            Ok(())
        }
        SearchOutcome::NodeLimit => Err("node limit reached".to_string()),
        SearchOutcome::NotFound => Err("no solution found".to_string()),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puzzle(width: usize, height: usize, letters: &str, words: &[&str]) -> Puzzle {
        Puzzle {
            name: "test".to_string(),
            width,
            height,
            letters: letters.to_string(),
            words: words.iter().map(|w| w.to_string()).collect(),
        }
    }

    fn solver(board: &Board, targets: &[&str], dict: &[&str]) -> Solver {
        Solver::new(
            board,
            targets.iter().map(|w| w.to_string()).collect(),
            Trie::new(dict.iter().map(|w| w.to_string()).collect::<Vec<_>>()),
        )
        .unwrap()
    }

    #[test]
    fn precomputes_compatible_paths() {
        let board = Board::from_puzzle(&puzzle(2, 2, "a___", &["abc"])).unwrap();
        let paths = precompute_paths(&board, "abc");

        assert!(!paths.is_empty());
        assert!(
            paths
                .iter()
                .all(|path| path_compatible(&board, "abc", path))
        );
        assert!(paths.iter().any(|path| path[0] == 0));
    }

    #[test]
    fn rejects_target_without_path() {
        let board = Board::from_puzzle(&puzzle(2, 2, "a!!!", &["ab"])).unwrap();
        let err = match Solver::new(
            &board,
            HashSet::from(["ab".to_string()]),
            Trie::new(vec!["ab".to_string()]),
        ) {
            Ok(_) => panic!("expected impossible target to fail"),
            Err(err) => err,
        };

        assert!(err.contains("no realizable path"));
    }

    #[test]
    fn detects_extra_dictionary_word() {
        let mut board = Board::from_puzzle(&puzzle(2, 2, "cat_", &["cat"])).unwrap();
        board.set(3, 's');
        let solver = solver(&board, &["cat"], &["cat", "cats"]);

        assert!(solver.has_extra_complete_word(&board));
    }

    #[test]
    fn path_assignment_rejects_conflicts_and_restores() {
        let mut board = Board::from_puzzle(&puzzle(2, 2, "a___", &["abc"])).unwrap();
        let changed = assign_path(&mut board, "abc", &[0, 1, 2]).unwrap();

        assert_eq!(changed, vec![1, 2]);
        assert_eq!(board.cells[1], Some('b'));
        assert_eq!(board.cells[2], Some('c'));

        assert!(assign_path(&mut board, "adc", &[0, 1, 2]).is_none());
        assert_eq!(board.cells[1], Some('b'));
        assert_eq!(board.cells[2], Some('c'));

        unassign_path(&mut board, &changed);
        assert_eq!(board.cells[0], Some('a'));
        assert_eq!(board.cells[1], None);
        assert_eq!(board.cells[2], None);
    }

    #[test]
    fn solves_small_puzzle() {
        let mut board = Board::from_puzzle(&puzzle(2, 2, "____", &["cat"])).unwrap();
        let solver = solver(&board, &["cat"], &["cat"]);

        assert!(solver.solve(&mut board));
        assert_eq!(
            solver.found_words(&board),
            HashSet::from(["cat".to_string()])
        );
    }

    #[test]
    fn word_path_search_records_word_nodes() {
        let mut board = Board::from_puzzle(&puzzle(2, 2, "____", &["cat"])).unwrap();
        let solver = solver(&board, &["cat"], &["cat"]);
        let mut stats = SearchStats::default();

        assert_eq!(
            solver.solve_with_stats(&mut board, &mut stats, None, SolverOptions::default()),
            SearchOutcome::Found
        );
        assert!(stats.word_nodes > 0);
        assert!(stats.path_attempts > 0);
    }
}
