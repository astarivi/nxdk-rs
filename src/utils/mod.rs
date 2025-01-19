use crate::utils::error::PlatformError;
use crate::winapi::WindowsPath;

pub mod error;

/// Converts a path &str to nul terminated char array; a cstr.
pub fn path_str_to_cstr(path_str: &str) -> Result<WindowsPath, PlatformError> {
    let mut path_buffer: [u8; 260] = [0; 260];
    let path_bytes = path_str.as_bytes();
    if path_bytes.len() > 259 {
        return Err(PlatformError::PathTooLong);
    }

    path_buffer[0..path_bytes.len()].copy_from_slice(path_bytes);

    if path_buffer[path_bytes.len() - 1] != 0 {
        path_buffer[path_bytes.len()] = 0;
    }

    Ok(path_buffer)
}
