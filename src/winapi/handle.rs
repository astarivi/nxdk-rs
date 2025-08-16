use core::ffi::c_void;
use nxdk_sys::winapi::{CloseHandle, ERROR_INVALID_HANDLE, HANDLE};
use crate::winapi::error::WinError;

pub const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

pub fn close_handle_native(handle: HANDLE) -> Result<(), WinError> {
    if handle.is_null() {
        // Should we error out here?
        return Ok(())
    }

    let result = unsafe {
        CloseHandle(handle)
    };

    if result == 0 {
        return Err(WinError::from_last_error())
    }

    Ok(())
}

#[derive(Debug)]
pub struct GenericWinHandle {
    handle: Option<HANDLE>
}

unsafe impl Send for GenericWinHandle {}

impl GenericWinHandle {
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle: Some(handle)
        }
    }

    pub fn get_inner(&self) -> Result<HANDLE, WinError> {
        self.handle.ok_or(WinError::from(ERROR_INVALID_HANDLE))
    }

    pub fn is_closed(&self) -> bool {
        !self.handle.is_some()
    }

    pub fn close(&mut self) -> Result<(), WinError> {
        if let Some(handle) = self.handle.take() {
            close_handle_native(handle)?;
        }

        Ok(())
    }
}