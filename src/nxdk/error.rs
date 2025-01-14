use core::error::Error;
use core::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NxNetError {
    InitError,
    DhcpError,
    UnknownError,
}

impl NxNetError {
    pub fn from_code(code: i32) -> Self {
        match code {
            -1 => NxNetError::InitError,
            -2 => NxNetError::DhcpError,
            _ => {
                NxNetError::UnknownError
            }
        }
    }
}

impl From<i32> for NxNetError {
    fn from(code: i32) -> Self {
        NxNetError::from_code(code)
    }
}

impl Display for NxNetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            NxNetError::InitError => write!(f, "Generic init error"),
            NxNetError::DhcpError => write!(f, "DHCP timeout"),
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

impl Error for NxNetError {}