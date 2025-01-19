pub mod error;
pub mod file;

pub type WindowsPath = [u8; 260];

use nxdk_sys::winapi::Sleep;

/// Sleep the current thread.
///
/// Time is in ms.
pub fn sleep(ms: u32) {
    unsafe {
        Sleep(ms);
    }
}