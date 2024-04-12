use thiserror::Error;
use std::string::FromUtf8Error;

#[derive(Debug, Error)]
pub enum Error{
    #[error("{0}")]
    Utf8Error(#[from]FromUtf8Error),
    #[error("{0}")]
    CsvError(#[from] csv::Error),
    #[error("{0}")]
    DataError(#[from]gauntlet::Error),
}
