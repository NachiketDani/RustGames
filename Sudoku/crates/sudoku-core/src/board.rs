use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ParseError;

pub const SIZE: usize = 9;
pub const CELLS: usize = 81;

/// A 9x9 Sudoku grid. `0` means empty; `1..=9` are digits.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    cells: [u8; CELLS],
}

pub fn row_of(idx: usize) -> usize {
    idx / SIZE
}

pub fn col_of(idx: usize) -> usize {
    idx % SIZE
}

pub fn box_of(idx: usize) -> usize {
    (row_of(idx) / 3) * 3 + col_of(idx) / 3
}

impl Board {
    pub fn empty() -> Self {
        Self { cells: [0; CELLS] }
    }

    pub fn from_cells(cells: [u8; CELLS]) -> Option<Self> {
        cells.iter().all(|&d| d <= 9).then_some(Self { cells })
    }

    pub fn cells(&self) -> &[u8; CELLS] {
        &self.cells
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.cells[row * SIZE + col]
    }

    /// Set a cell without checking Sudoku rules. Panics if `digit > 9` or the
    /// position is out of range.
    pub fn set(&mut self, row: usize, col: usize, digit: u8) {
        assert!(row < SIZE && col < SIZE && digit <= 9);
        self.cells[row * SIZE + col] = digit;
    }

    pub fn clear(&mut self, row: usize, col: usize) {
        self.set(row, col, 0);
    }

    pub fn clue_count(&self) -> usize {
        self.cells.iter().filter(|&&d| d != 0).count()
    }

    pub fn is_full(&self) -> bool {
        self.cells.iter().all(|&d| d != 0)
    }

    /// Would placing `digit` at (row, col) clash with another cell in its
    /// row, column or box? The cell itself is ignored.
    pub fn can_place(&self, row: usize, col: usize, digit: u8) -> bool {
        let idx = row * SIZE + col;
        (0..CELLS)
            .filter(|&i| i != idx)
            .filter(|&i| row_of(i) == row || col_of(i) == col || box_of(i) == box_of(idx))
            .all(|i| self.cells[i] != digit)
    }

    /// Indices of all filled cells that share a digit with a peer.
    pub fn conflicts(&self) -> Vec<usize> {
        (0..CELLS)
            .filter(|&i| {
                let d = self.cells[i];
                d != 0 && !self.can_place(row_of(i), col_of(i), d)
            })
            .collect()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts().is_empty()
    }

    /// Full and conflict-free.
    pub fn is_solved(&self) -> bool {
        self.is_full() && !self.has_conflicts()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::empty()
    }
}

impl FromStr for Board {
    type Err = ParseError;

    /// Accepts digits 1-9 and `.` or `0` for empty. Whitespace is ignored.
    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut cells = Vec::with_capacity(CELLS);
        for ch in s.chars().filter(|c| !c.is_whitespace()) {
            match ch {
                '.' | '0' => cells.push(0),
                '1'..='9' => cells.push(ch as u8 - b'0'),
                other => return Err(ParseError::InvalidChar(other)),
            }
        }
        let len = cells.len();
        let cells: [u8; CELLS] = cells.try_into().map_err(|_| ParseError::WrongLength(len))?;
        Ok(Self { cells })
    }
}

/// Compact 81-character form, `.` for empty.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &d in &self.cells {
            let ch = if d == 0 { '.' } else { (b'0' + d) as char };
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}

/// Human-friendly grid with box separators.
impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..SIZE {
            if r % 3 == 0 && r != 0 {
                writeln!(f, "------+-------+------")?;
            }
            for c in 0..SIZE {
                if c % 3 == 0 && c != 0 {
                    write!(f, "| ")?;
                }
                let d = self.get(r, c);
                write!(f, "{} ", if d == 0 { '.' } else { (b'0' + d) as char })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Serialize for Board {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn parse_and_display_round_trip() {
        let b: Board = PUZZLE.parse().unwrap();
        assert_eq!(b.get(0, 0), 5);
        assert_eq!(b.get(0, 2), 0);
        assert_eq!(b.clue_count(), 30);
        assert_eq!(b.to_string().parse::<Board>().unwrap(), b);
    }

    #[test]
    fn parse_errors() {
        assert_eq!("123".parse::<Board>(), Err(ParseError::WrongLength(3)));
        let bad = "x".repeat(81);
        assert_eq!(bad.parse::<Board>(), Err(ParseError::InvalidChar('x')));
    }

    #[test]
    fn detects_conflicts() {
        let mut b: Board = PUZZLE.parse().unwrap();
        assert!(!b.has_conflicts());
        b.set(0, 2, 5); // duplicates the 5 at (0,0)
        assert!(b.has_conflicts());
        assert!(b.conflicts().contains(&0) && b.conflicts().contains(&2));
        assert!(!b.can_place(0, 3, 5));
        assert!(!b.can_place(0, 3, 1)); // 1 is already in column 3
        assert!(b.can_place(0, 3, 6));
    }

    #[test]
    fn serde_uses_string_form() {
        let b: Board = PUZZLE.parse().unwrap();
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(json, format!("\"{}\"", PUZZLE.replace('0', ".")));
        assert_eq!(serde_json::from_str::<Board>(&json).unwrap(), b);
    }
}
