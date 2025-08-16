use core::error::Error;
use core::fmt;
use core::fmt::{Display, Formatter};
use nxdk_sys::lwip::*;

/// Error type for Netconn APIs. Names equivalent to enum names. Can be cast to the
/// error number for inter-op.
///
/// Panics if the error number is outside the Netconn range.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NetconnErr {
    Ok = err_enum_t_ERR_OK as isize,
    Mem = err_enum_t_ERR_MEM as isize,
    Buf = err_enum_t_ERR_BUF as isize,
    Timeout = err_enum_t_ERR_TIMEOUT as isize,
    Rte = err_enum_t_ERR_RTE as isize,
    InProgress = err_enum_t_ERR_INPROGRESS as isize,
    Val = err_enum_t_ERR_VAL as isize,
    WouldBlock = err_enum_t_ERR_WOULDBLOCK as isize,
    Use = err_enum_t_ERR_USE as isize,
    Already = err_enum_t_ERR_ALREADY as isize,
    IsConn = err_enum_t_ERR_ISCONN as isize,
    Conn = err_enum_t_ERR_CONN as isize,
    If = err_enum_t_ERR_IF as isize,
    Abrt = err_enum_t_ERR_ABRT as isize,
    Rst = err_enum_t_ERR_RST as isize,
    Clsd = err_enum_t_ERR_CLSD as isize,
    Arg = err_enum_t_ERR_ARG as isize
}

impl NetconnErr {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => NetconnErr::Ok,
            -1 => NetconnErr::Mem,
            -2 => NetconnErr::Buf,
            -3 => NetconnErr::Timeout,
            -4 => NetconnErr::Rte,
            -5 => NetconnErr::InProgress,
            -6 => NetconnErr::Val,
            -7 => NetconnErr::WouldBlock,
            -8 => NetconnErr::Use,
            -9 => NetconnErr::Already,
            -10 => NetconnErr::IsConn,
            -11 => NetconnErr::Conn,
            -12 => NetconnErr::If,
            -13 => NetconnErr::Abrt,
            -14 => NetconnErr::Rst,
            -15 => NetconnErr::Clsd,
            -16 => NetconnErr::Arg,
            _ => {
                // Handle any other unexpected values
                panic!("Unknown Netconn error code: {}", code);
            }
        }
    }
}

impl From<i32> for NetconnErr {
    fn from(code: i32) -> Self {
        NetconnErr::from_code(code)
    }
}

impl From<i8> for NetconnErr {
    fn from(code: i8) -> Self {
        NetconnErr::from_code(code as i32)
    }
}

impl Error for NetconnErr {}

impl Display for NetconnErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl embedded_io::Error for NetconnErr {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}