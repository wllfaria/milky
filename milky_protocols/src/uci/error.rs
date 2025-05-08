use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    InsufficientCommand(String),
    #[error("{0}")]
    UnknownCommand(&'static str),
    #[error("{0}")]
    InvalidCommand(String),
    #[error("{0}")]
    Fen(#[from] milky_fen::Error),
}

pub type Result<R> = std::result::Result<R, Error>;
