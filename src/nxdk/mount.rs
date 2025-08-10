use crate::nxdk::path::nx_get_current_xbe_nt_path_native;
use crate::utils::error::PlatformError;
use crate::utils::path_str_to_cstr;
use nxdk_sys::nxdk::mount::{nxIsDriveMounted, nxMountDrive, nxUnmountDrive};

/// Mounts the given path to the given drive letter.
///
/// Returns a bool for result status, or PlatformError::PathTooLong if a given
/// path is over 259 characters.
pub fn nx_mount_drive(drive_letter: char, path: &str) -> Result<bool, PlatformError> {
    let c_path = path_str_to_cstr(path)?;

    Ok(unsafe { nxMountDrive(drive_letter as u8 as i8, c_path.as_ptr() as *const i8) })
}

/// Helper method to mount the current execution path to the given
/// drive letter.
///
/// This is the equivalent of mounting `nxGetCurrentXbeNtPath` result
/// to the given drive.
pub fn nx_mount_execution_to(drive_letter: char) -> bool {
    nx_mount_drive_from(drive_letter, &mut nx_get_current_xbe_nt_path_native())
}

fn nx_mount_drive_from(drive_letter: char, native_path: &mut [libc::c_char; 260]) -> bool {
    if let Some(last_backslash) = native_path
        .iter()
        .rposition(|&c| c == b'\\' as libc::c_char)
    {
        if last_backslash < 260 {
            native_path[last_backslash] = 0;
        }
    }

    unsafe { nxMountDrive(drive_letter as u8 as i8, native_path.as_ptr() as *const i8) }
}

pub fn nx_unmount_drive(drive_letter: char) -> bool {
    unsafe { nxUnmountDrive(drive_letter as u8 as i8) }
}

pub fn nx_is_drive_mounted(drive_letter: char) -> bool {
    unsafe { nxIsDriveMounted(drive_letter as u8 as i8) }
}
