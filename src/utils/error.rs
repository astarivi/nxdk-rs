use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PlatformError {
    PathTooLong,
}

impl Display for PlatformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PlatformError::PathTooLong => write!(f, "This path is longer than 259 characters."),
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

impl Error for PlatformError {}