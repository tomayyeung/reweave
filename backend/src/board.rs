/// A cell of the board, indexed by its coordinates
#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub struct BoardCell(pub usize, pub usize);

/// A board of letters, some of which might not be filled in
#[allow(unused)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Option<char>>>,
}

impl Board {
    /// Create a Board given a width, height, and a vector of characters
    /// Panics if the length of chars does not match width * height
    /// For an empty cell, pass in ' '
    pub fn create(width: usize, height: usize, chars: Vec<char>) -> Self {
        assert_eq!(width * height, chars.len());

        let mut cells: Vec<Vec<Option<char>>> = Vec::new();
        let mut i = 0;
        for _ in 0..height {
            let mut row = Vec::new();

            for _ in 0..width {
                let c = chars.get(i).unwrap();

                // Empty cell
                if *c == ' ' {
                    row.push(None);
                }

                // Check if valid char; if so, add to board
                if c.is_ascii_lowercase() {
                    row.push(Some(*c))
                } else {
                    panic!("Invalid character when creating board {c}");
                }

                i += 1;
            }

            cells.push(row);
        }

        Board {
            width,
            height,
            cells,
        }
    }

    pub fn get(&self, cell: BoardCell) -> Option<char> {
        *self.cells.get(cell.0)?.get(cell.1)?
    }
}
