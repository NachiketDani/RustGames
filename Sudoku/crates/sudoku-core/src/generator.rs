use serde::{Deserialize, Serialize};

use crate::board::{Board, CELLS};
use crate::rng::Rng;
use crate::solver::{has_unique_solution, random_solution};

/// Difficulty is currently approximated by how many clues are left.
/// A technique-based rating (singles, pairs, X-Wing, ...) will replace this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    /// Remove cells until at most this many clues remain (if uniqueness allows).
    pub fn target_clues(self) -> usize {
        match self {
            Difficulty::Easy => 38,
            Difficulty::Medium => 32,
            Difficulty::Hard => 28,
            Difficulty::Expert => 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    pub seed: u64,
    pub difficulty: Difficulty,
    /// The starting grid (givens only). Has exactly one solution.
    pub puzzle: Board,
    pub solution: Board,
}

/// Generate a puzzle. The same `(seed, difficulty)` always gives the same result.
pub fn generate(seed: u64, difficulty: Difficulty) -> Puzzle {
    let mut rng = Rng::new(seed);
    let solution = random_solution(rng.clone());

    let mut order: Vec<usize> = (0..CELLS).collect();
    rng.shuffle(&mut order);

    let mut puzzle = solution;
    let mut clues = CELLS;
    for idx in order {
        if clues <= difficulty.target_clues() {
            break;
        }
        let (r, c) = (idx / 9, idx % 9);
        let digit = puzzle.get(r, c);
        puzzle.clear(r, c);
        if has_unique_solution(&puzzle) {
            clues -= 1;
        } else {
            puzzle.set(r, c, digit);
        }
    }

    Puzzle {
        seed,
        difficulty,
        puzzle,
        solution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::solve;

    #[test]
    fn generated_puzzles_are_unique_and_match_solution() {
        for (seed, diff) in [
            (1, Difficulty::Easy),
            (2, Difficulty::Medium),
            (3, Difficulty::Hard),
            (4, Difficulty::Expert),
        ] {
            let p = generate(seed, diff);
            assert!(has_unique_solution(&p.puzzle), "seed {seed} not unique");
            assert_eq!(solve(&p.puzzle), Some(p.solution));
            assert!(p.solution.is_solved());
            assert!(p.puzzle.clue_count() <= diff.target_clues() + 6);
        }
    }

    #[test]
    fn harder_means_fewer_clues() {
        let easy = generate(10, Difficulty::Easy).puzzle.clue_count();
        let expert = generate(10, Difficulty::Expert).puzzle.clue_count();
        assert!(expert < easy);
    }

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(generate(42, Difficulty::Hard), generate(42, Difficulty::Hard));
        assert_ne!(
            generate(42, Difficulty::Hard).puzzle,
            generate(43, Difficulty::Hard).puzzle
        );
    }
}
