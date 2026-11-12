use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("expected 81 cells, found {0}")]
    WrongLength(usize),
    #[error("invalid character {0:?} (use 1-9, or . / 0 for empty)")]
    InvalidChar(char),
}
