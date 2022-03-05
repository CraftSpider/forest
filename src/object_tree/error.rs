
use core::fmt;
use core::cell::{BorrowError, BorrowMutError};
#[cfg(feature = "std")]
use std::error::Error as StdError;

pub(super) type Result<T> = core::result::Result<T, Error>;

/// Possible failures for tree operations
#[derive(Debug)]
pub enum Error {
    /// Node doesn't exist
    Missing,
    /// Node can't be borrowed as requested
    CantBorrow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Missing => write!(f, "Tree missing expected node"),
            Error::CantBorrow => write!(f, "Tree node is already borrowed incompatibly"),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {}

impl From<BorrowError> for Error {
    fn from(_: BorrowError) -> Self {
        Error::CantBorrow
    }
}

impl From<BorrowMutError> for Error {
    fn from(_: BorrowMutError) -> Self {
        Error::CantBorrow
    }
}
