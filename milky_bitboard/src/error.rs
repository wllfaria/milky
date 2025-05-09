use thiserror::Error;

pub type Result<R> = std::result::Result<R, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidSquare(String),
    #[error("{0}")]
    InvalidPiece(String),
}
