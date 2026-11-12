use crate::board::{box_of, col_of, row_of, Board, CELLS, SIZE};
use crate::rng::Rng;

const ALL: u16 = 0x1FF; // bits 0..=8 for digits 1..=9

/// Backtracking solver using row/column/box bitmasks and the
/// "minimum remaining values" heuristic (always branch on the most
/// constrained empty cell).
struct Search {
    cells: [u8; CELLS],
    rows: [u16; SIZE],
    cols: [u16; SIZE],
    boxes: [u16; SIZE],
    rng: Option<Rng>,
    limit: usize,
    count: usize,
    first: Option<[u8; CELLS]>,
}

impl Search {
    /// Returns `None` if the givens already conflict.
    fn new(board: &Board, rng: Option<Rng>, limit: usize) -> Option<Self> {
        let mut s = Self {
            cells: [0; CELLS],
            rows: [0; SIZE],
            cols: [0; SIZE],
            boxes: [0; SIZE],
            rng,
            limit,
            count: 0,
            first: None,
        };
        for (idx, &d) in board.cells().iter().enumerate() {
            if d == 0 {
                continue;
            }
            let bit = 1u16 << (d - 1);
            if (s.rows[row_of(idx)] | s.cols[col_of(idx)] | s.boxes[box_of(idx)]) & bit != 0 {
                return None;
            }
            s.place(idx, d);
        }
        Some(s)
    }

    fn place(&mut self, idx: usize, d: u8) {
        let bit = 1u16 << (d - 1);
        self.cells[idx] = d;
        self.rows[row_of(idx)] |= bit;
        self.cols[col_of(idx)] |= bit;
        self.boxes[box_of(idx)] |= bit;
    }

    fn unplace(&mut self, idx: usize, d: u8) {
        let bit = !(1u16 << (d - 1));
        self.cells[idx] = 0;
        self.rows[row_of(idx)] &= bit;
        self.cols[col_of(idx)] &= bit;
        self.boxes[box_of(idx)] &= bit;
    }

    fn candidates(&self, idx: usize) -> u16 {
        !(self.rows[row_of(idx)] | self.cols[col_of(idx)] | self.boxes[box_of(idx)]) & ALL
    }

    /// Most constrained empty cell, or `None` if the grid is full.
    fn pick_cell(&self) -> Option<(usize, u16)> {
        let mut best: Option<(usize, u16)> = None;
        for idx in (0..CELLS).filter(|&i| self.cells[i] == 0) {
            let cands = self.candidates(idx);
            let n = cands.count_ones();
            if n == 0 {
                return Some((idx, 0));
            }
            if best.is_none_or(|(_, b)| n < b.count_ones()) {
                best = Some((idx, cands));
            }
        }
        best
    }

    fn run(&mut self) {
        if self.count >= self.limit {
            return;
        }
        let Some((idx, cands)) = self.pick_cell() else {
            self.count += 1;
            self.first.get_or_insert(self.cells);
            return;
        };
        if cands == 0 {
            return;
        }

        let mut digits = [0u8; 9];
        let mut n = 0;
        for d in 1..=9u8 {
            if cands & (1 << (d - 1)) != 0 {
                digits[n] = d;
                n += 1;
            }
        }
        if let Some(rng) = self.rng.as_mut() {
            rng.shuffle(&mut digits[..n]);
        }

        for &d in &digits[..n] {
            self.place(idx, d);
            self.run();
            self.unplace(idx, d);
            if self.count >= self.limit {
                return;
            }
        }
    }
}

/// Find one solution, or `None` if the puzzle has none.
pub fn solve(board: &Board) -> Option<Board> {
    let mut s = Search::new(board, None, 1)?;
    s.run();
    s.first.and_then(Board::from_cells)
}

/// Count solutions, stopping once `limit` is reached.
pub fn count_solutions(board: &Board, limit: usize) -> usize {
    match Search::new(board, None, limit) {
        Some(mut s) => {
            s.run();
            s.count
        }
        None => 0,
    }
}

pub fn has_unique_solution(board: &Board) -> bool {
    count_solutions(board, 2) == 1
}

/// A random completed grid for the given seed (deterministic).
pub(crate) fn random_solution(rng: Rng) -> Board {
    let mut s = Search::new(&Board::empty(), Some(rng), 1).expect("empty board is valid");
    s.run();
    s.first
        .and_then(Board::from_cells)
        .expect("an empty board always has a solution")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn solves_known_puzzle() {
        let p: Board = PUZZLE.parse().unwrap();
        let s = solve(&p).unwrap();
        assert_eq!(s.to_string(), SOLUTION);
        assert!(s.is_solved());
    }

    #[test]
    fn known_puzzle_is_unique() {
        let p: Board = PUZZLE.parse().unwrap();
        assert!(has_unique_solution(&p));
    }

    #[test]
    fn empty_board_has_many_solutions() {
        assert_eq!(count_solutions(&Board::empty(), 5), 5);
    }

    #[test]
    fn conflicting_givens_have_no_solution() {
        let mut p: Board = PUZZLE.parse().unwrap();
        p.set(0, 2, 5);
        assert_eq!(solve(&p), None);
        assert_eq!(count_solutions(&p, 2), 0);
    }

    #[test]
    fn unsolvable_without_conflict() {
        // Row 0 holds 1-8 so (0,8) needs 9, but 9 is already in column 8.
        let p: Board = format!("12345678.{}{}", ".".repeat(8) + "9", ".".repeat(63))
            .parse()
            .unwrap();
        assert!(!p.has_conflicts());
        assert_eq!(solve(&p), None);
    }

    #[test]
    fn random_solution_is_valid_and_seeded() {
        let a = random_solution(Rng::new(1));
        let b = random_solution(Rng::new(1));
        let c = random_solution(Rng::new(2));
        assert!(a.is_solved());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
