use crate::utils::error::PlatformError;
use crate::utils::path_str_to_cstr;
use nxdk_sys::nxdk::format::nxFormatVolume;

/// Formats a volume with FATX.
/// WARNING: This is a destructive operation, incorrect use can lead to unexpected data loss!
///
/// Returns true on success. When false, additional error information is available from
/// GetLastError()
///
/// # Arguments
///  - `volume_path` The NT-style path to the volume that is about to be formatted
///  - `bytes_per_cluster` Specifies the number of bytes per cluster, pass 0 for default
pub fn nx_format_volume(volume_path: &str, bytes_per_cluster: u32) -> Result<bool, PlatformError> {
    let c_path = path_str_to_cstr(volume_path)?;

    Ok(unsafe { nxFormatVolume(c_path.as_ptr() as *const i8, bytes_per_cluster) })
}
