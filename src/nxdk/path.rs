use alloc::string::{String, ToString};
use core::ffi::CStr;
use core::str::Utf8Error;
use nxdk_sys::nxdk::path::nxGetCurrentXbeNtPath;

pub fn nx_get_current_xbe_nt_path_native() -> [libc::c_char; 260] {
    let mut target_path = [0 as libc::c_char; 260];

    unsafe {
        nxGetCurrentXbeNtPath(target_path.as_mut_ptr());
    }

    target_path
}

pub fn nx_get_current_xbe_nt_path() -> Result<String, Utf8Error> {
    let target_path = nx_get_current_xbe_nt_path_native();

    let c_string = unsafe { CStr::from_ptr(target_path.as_ptr() as *const i8) };

    Ok(c_string.to_str()?.to_string())
}
