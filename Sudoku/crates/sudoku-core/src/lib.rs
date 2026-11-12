//! Core Sudoku logic: board, solver, and puzzle generator.

pub mod board;
pub mod error;
pub mod generator;
pub mod rng;
pub mod solver;

pub use board::Board;
pub use error::ParseError;
pub use generator::{generate, Difficulty, Puzzle};
pub use solver::{count_solutions, has_unique_solution, solve};
