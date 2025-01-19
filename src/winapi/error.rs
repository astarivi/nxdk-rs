use core::error::Error;
use core::fmt::{Display, Formatter};
use nxdk_sys::winapi::GetLastError;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WinError(u32);

impl WinError {
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    pub fn from_last_error() -> Self {
        Self::new(unsafe { GetLastError() })
    }

    pub const fn into_inner(self) -> u32 {
        self.0
    }
}

impl Display for WinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Win error code: {}", self.0)
    }
}

impl Error for WinError {}

impl From<u32> for WinError {
    fn from(o: u32) -> Self {
        Self::new(o)
    }
}

impl From<WinError> for u32 {
    fn from(o: WinError) -> Self {
        o.into_inner()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NtStatusError(i32);

impl NtStatusError {
    pub const fn new(code: i32) -> Self {
        Self(code)
    }

    pub const fn into_inner(self) -> i32 {
        self.0
    }
}

impl Display for NtStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NTSTATUS error code: {}", self.0)
    }
}

impl Error for NtStatusError {}

impl From<i32> for NtStatusError {
    fn from(o: i32) -> Self {
        Self::new(o)
    }
}


/// A Win Mixed error represents an error that could either be
/// a WinError, or a NtStatusError
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WinMixedError {
    NtStatus(NtStatusError),
    WinError(WinError),
}

impl Display for WinMixedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            WinMixedError::NtStatus(nt_status_error) => {
                write!(f, "NtStatus error code: {}", nt_status_error)
            }
            WinMixedError::WinError(win_error) => {
                write!(f, "WinError code: {}", win_error)
            }
        }
    }
}

impl Error for WinMixedError {}