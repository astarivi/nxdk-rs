use crate::utils::error::PlatformError;
use crate::utils::path_str_to_cstr;
use nxdk_sys::hal::xbox::{XLaunchXBE, XReboot};

/// Reboot the console. Duh. This shouldn't return.
pub fn xreboot() {
    unsafe {
        XReboot();
    }
}

/// Launches an XBE.
/// 
/// If the XBE is able to launch, this method will not return. Otherwise, returns like normal.
/// 
/// # Examples of xbe_path:
/// - `c:\blah.xbe`
/// - `c:/foo/bar.xbe`
pub fn xlaunch_xbe(xbe_path: &str) -> Result<(), PlatformError>{
    let c_path = path_str_to_cstr(xbe_path)?;

    unsafe {
        XLaunchXBE(
            c_path.as_ptr() as *const i8,
        );
    }

    Ok(())
}