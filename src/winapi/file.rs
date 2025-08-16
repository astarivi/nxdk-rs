use crate::winapi::error::{NtStatusError, WinError, WinMixedError};
use crate::winapi::handle::{close_handle_native, GenericWinHandle};
use crate::winapi::WindowsPath;
use bitflags::bitflags;
use core::ffi::c_void;
use embedded_io::SeekFrom;
use futures_lite::future::yield_now;
use log::error;
use nxdk_sys::winapi::*;

pub const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AccessRights: u32 {
        const Read = GENERIC_READ;
        const Write = GENERIC_WRITE;
        const Execute = GENERIC_EXECUTE;
        const All = GENERIC_ALL;
        const None = 0;
    }
}

impl Default for AccessRights{
    fn default() -> Self {
        Self::Read
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ShareMode: u32 {
        const Read = FILE_SHARE_READ;
        const Write = FILE_SHARE_WRITE;
        const Delete = FILE_SHARE_DELETE;
        /// Doesn't share the file; exclusive access.
        const None = 0;
    }
}

impl Default for ShareMode {
    fn default() -> Self {
        Self::Read | Self::Write
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub enum CreationDisposition {
    CreateAlways = CREATE_ALWAYS as isize,
    CreateNew = CREATE_NEW as isize,
    OpenAlways = OPEN_ALWAYS as isize,
    #[default]
    OpenExisting = OPEN_EXISTING as isize,
    TruncateExisting = TRUNCATE_EXISTING as isize,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FileFlagsAndAttributes: u32 {
        const AttributeArchive = FILE_ATTRIBUTE_ARCHIVE;
        const AttributeHidden = FILE_ATTRIBUTE_HIDDEN;
        const AttributeNormal = FILE_ATTRIBUTE_NORMAL;
        const AttributeReadOnly = FILE_ATTRIBUTE_READONLY;
        const AttributeSystem = FILE_ATTRIBUTE_SYSTEM;
        const AttributeTemporary = FILE_ATTRIBUTE_TEMPORARY;
        const FlagBackupSemantics = FILE_FLAG_BACKUP_SEMANTICS;
        const FlagDeleteOnClose = FILE_FLAG_DELETE_ON_CLOSE;
        const FlagNoBuffering = FILE_FLAG_NO_BUFFERING;
        const FlagOverlapped = FILE_FLAG_OVERLAPPED;
        const FlagPosixSemantics = FILE_FLAG_POSIX_SEMANTICS;
        const FlagRandomAccess = FILE_FLAG_RANDOM_ACCESS;
        const FlagSequentialScan = FILE_FLAG_SEQUENTIAL_SCAN;
        const FlagWriteThrough = FILE_FLAG_WRITE_THROUGH;
    }
}

impl Default for FileFlagsAndAttributes {
    fn default() -> Self {
        Self::AttributeNormal
    }
}

pub struct FileStandardInformation {
    pub allocation_size: u64,
    /// Also known as filesize
    pub end_of_file: u64,
    pub number_of_links: u32,
    pub delete_pending: bool,
    pub directory: bool
}

#[derive(Debug)]
pub struct Overlapped {
    overlapped: Option<OVERLAPPED>,
    offset: u64
}

impl Overlapped {
    pub fn wrap(overlapped: Option<OVERLAPPED>) -> Self {
        Self {
            overlapped,
            offset: 0
        }
    }

    pub fn new() -> Result<Self, WinError> {
        let overlapped_handle = unsafe {
            CreateEventA(
                core::ptr::null_mut(),
                true as i32,
                false as i32,
                core::ptr::null()
            )
        };

        if overlapped_handle.is_null() {
            return Err(WinError::from_last_error())
        }

        Ok(Self {
            overlapped: Some(OVERLAPPED {
                Internal: 0,
                InternalHigh: 0,
                Offset: 0,
                OffsetHigh: 0,
                hEvent: overlapped_handle
            }),
            offset: 0
        })
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    pub fn advance_offset(&mut self, offset: i64) {
        let new_offset = self.offset as i64 + offset;
        self.offset = new_offset.max(0) as u64;
    }

    pub fn get_inner(&mut self) -> Result<&mut OVERLAPPED, WinError> {
        self.overlapped.as_mut().ok_or(WinError::from(ERROR_INVALID_HANDLE))
    }

    pub fn is_closed(&self) -> bool {
        !self.overlapped.is_some()
    }

    pub fn reset_overlapped(&mut self) -> Result<(), WinError> {
        let offset = self.offset;
        let overlapped = self.get_inner()?;

        let result = unsafe {
            ResetEvent(overlapped.hEvent)
        };

        if result == 0 {
            return Err(WinError::from_last_error())
        }

        overlapped.Offset = offset as u32;
        overlapped.OffsetHigh = (offset >> 32) as u32;
        overlapped.Internal = 0;
        overlapped.InternalHigh = 0;

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), WinError> {
        if let Some(mut overlapped) = self.overlapped.take() {
            close_handle_native(overlapped.hEvent)?;
            // Release it just in case
            overlapped.hEvent = core::ptr::null_mut();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct WinFileHandle {
    handle: GenericWinHandle,
    overlapped: Option<Overlapped>
}

unsafe impl Send for WinFileHandle {}

impl WinFileHandle {
    /// Be careful to never initialize this with a search HANDLE. Doing so
    /// will result in unexpected panic.
    ///
    /// Also, if overlapped is given, it must not be null.
    pub fn new(handle: GenericWinHandle, overlapped: Option<Overlapped>) -> Self {
        WinFileHandle {
            handle,
            overlapped
        }
    }

    pub fn open(path: &WindowsPath, access: AccessRights, share: ShareMode, creation: CreationDisposition, flags_attributes: FileFlagsAndAttributes) -> Result<Self, WinError> {
        let handle = unsafe {
            CreateFileA(
                path.as_ptr() as *const i8,
                access.bits(),
                share.bits(),
                core::ptr::null_mut(),
                creation as u32,
                flags_attributes.bits(),
                core::ptr::null_mut()
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(WinError::from_last_error())
        }

        Ok(
            Self {
                handle: GenericWinHandle::new(handle),
                overlapped: if flags_attributes.contains(FileFlagsAndAttributes::FlagOverlapped) {
                    Some(Overlapped::new()?)
                } else {
                    None
                }
            }
        )
    }

    fn get_inner(&self) -> Result<HANDLE, WinError> {
        self.handle.get_inner()
    }

    fn get_overlapped(&mut self) -> Result<&mut Overlapped, WinError> {
        if let Some(overlapped) = self.overlapped.as_mut() {
            Ok(overlapped)
        } else {
            error!(
                "Tried to get overlapped from a non-overlapped file handle. Use flag \
                FileFlagsAndAttributes::FlagOverlapped when opening the file."
            );

            Err(WinError::from(ERROR_INVALID_HANDLE))
        }

    }

    pub fn reset_overlapped(&mut self) -> Result<(), WinError> {
        if let Some(overlapped) = self.overlapped.as_mut() {
            overlapped.reset_overlapped()?;
        }

        Ok(())
    }

    /// Query standard handle information. This can be called from
    /// any open handle, regardless of the `AccessRights` mode.
    pub fn query_standard_information(&self) -> Result<FileStandardInformation, WinMixedError> {
        let mut file_info = FILE_STANDARD_INFORMATION {
            AllocationSize: LARGE_INTEGER { QuadPart: 0 },
            EndOfFile: LARGE_INTEGER { QuadPart: 0 },
            NumberOfLinks: 0,
            DeletePending: 0,
            Directory: 0,
        };

        let status = unsafe {
            NtQueryInformationFile(
                self.get_inner().map_err(|err| WinMixedError::WinError(err))?,
                core::ptr::null_mut(),
                &mut file_info as *mut _ as *mut c_void,
                size_of::<FILE_STANDARD_INFORMATION>() as u32,
                _FILE_INFORMATION_CLASS_FileStandardInformation,
            )
        };

        if status != 0 {
            return Err(WinMixedError::NtStatus(NtStatusError::new(status)))
        }

        unsafe {
            Ok(FileStandardInformation {
                allocation_size: file_info.AllocationSize.QuadPart as u64,
                end_of_file: file_info.EndOfFile.QuadPart as u64,
                number_of_links: file_info.NumberOfLinks,
                delete_pending: file_info.DeletePending != 0,
                directory: file_info.Directory != 0,
            })
        }
    }

    pub fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    pub fn close(&mut self) -> Result<(), WinError> {
        self.handle.close()?;

        if let Some(mut overlapped) = self.overlapped.take() {
            overlapped.close()?;
        }

        Ok(())
    }
}

impl Drop for WinFileHandle {
    fn drop(&mut self) {
        if let Err(e) = self.handle.close() {
            error!("Error closing dropped file handle: {}", e);
        }
    }
}

impl embedded_io::ErrorType for WinFileHandle {
    type Error = WinError;
}

impl embedded_io::Write for WinFileHandle {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut bytes_written: u32 = 0;
        let success = unsafe {
            WriteFile(
                self.get_inner()?,
                buf.as_ptr() as *const c_void,
                buf.len() as u32,
                &mut bytes_written,
                core::ptr::null_mut(),
            )
        };

        if success == 0 {
            return Err(WinError::from_last_error())
        }

        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        panic!("Flushing WinNT file handles causes a deadlock")
    }
}

impl embedded_io::Read for WinFileHandle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut bytes_read: u32 = 0;
        let success = unsafe {
            ReadFile(
                self.get_inner()?,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut bytes_read,
                core::ptr::null_mut(),
            )
        };

        if success == 0 {
            return Err(WinError::from_last_error())
        }

        Ok(bytes_read as usize)
    }
}

impl embedded_io::Seek for WinFileHandle {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let (offset, move_method) = match pos {
            SeekFrom::Start(offset) => {
                (offset as i64, FILE_BEGIN)
            }
            SeekFrom::End(offset) => {
                (offset, FILE_END)
            }
            SeekFrom::Current(offset) => {
                (offset, FILE_CURRENT)
            }
        };

        let mut new_position = LARGE_INTEGER { QuadPart: offset };
        let success = unsafe {
            SetFilePointerEx(
                self.get_inner()?,
                new_position,
                &mut new_position,
                move_method
            )
        };

        if success == 0 {
            return Err(WinError::from_last_error())
        }

        unsafe {
            Ok(new_position.QuadPart as u64)
        }
    }
}

impl embedded_io_async::Write for WinFileHandle {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let handle = self.get_inner()?;
        self.reset_overlapped()?;
        let overlapped = self.get_overlapped()?;
        let inner_overlapped = overlapped.get_inner()?;

        let mut bytes_written: u32 = 0;
        let success = unsafe {
            WriteFile(
                handle,
                buf.as_ptr() as *const c_void,
                buf.len() as u32,
                &mut bytes_written,
                inner_overlapped,
            )
        };

        if success != 0 {
            overlapped.advance_offset(bytes_written as i64);
            return Ok(bytes_written as usize)
        }

        let error = WinError::from_last_error();
        if u32::from(error) != ERROR_IO_PENDING {
            return Err(error);
        }

        let mut overlapped_bytes_written: u32 = 0;
        loop {
            let result = unsafe {
                GetOverlappedResult(
                    handle,
                    inner_overlapped,
                    &mut overlapped_bytes_written,
                    0
                )
            };

            if result != 0 {
                overlapped.advance_offset(overlapped_bytes_written as i64);
                return Ok(overlapped_bytes_written as usize);
            }

            let last_error = WinError::from_last_error();

            if u32::from(last_error) != ERROR_IO_INCOMPLETE {
                return Err(last_error);
            }

            yield_now().await;
        }
    }
}

impl embedded_io_async::Read for WinFileHandle {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let handle = self.get_inner()?;
        self.reset_overlapped()?;
        let overlapped = self.get_overlapped()?;
        let inner_overlapped = overlapped.get_inner()?;

        let mut bytes_read: u32 = 0;

        let success = unsafe {
            ReadFile(
                handle,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut bytes_read,
                inner_overlapped,
            )
        };

        // Done immediately
        if success == 1 {
            overlapped.advance_offset(bytes_read as i64);
            return Ok(bytes_read as usize)
        }

        let error = WinError::from_last_error();
        if u32::from(error) != ERROR_IO_PENDING {
            return Err(error);
        }

        let mut overlapped_bytes_read: u32 = 0;
        loop {
            let result = unsafe {
                GetOverlappedResult(
                    handle,
                    inner_overlapped,
                    &mut overlapped_bytes_read,
                    0
                )
            };

            if result != 0 {
                overlapped.advance_offset(overlapped_bytes_read as i64);
                return Ok(overlapped_bytes_read as usize);
            }

            let last_error = WinError::from_last_error();

            if u32::from(last_error) != ERROR_IO_INCOMPLETE {
                return Err(last_error);
            }

            yield_now().await;
        }
    }
}

impl embedded_io_async::Seek for WinFileHandle {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let (offset) = match pos {
            SeekFrom::Start(offset) => {
                offset as i64
            },
            SeekFrom::Current(offset) => {
                offset
            },
            _ => {
                error!(
                    "Seeking from the End is not supported in async file handles"
                );

                return Err(WinError::from(ERROR_INVALID_PARAMETER))
            }
        };

        if let Some(overlapped) = self.overlapped.as_mut() {
            overlapped.advance_offset(offset);
            Ok(overlapped.offset)
        } else {
            error!(
                "Tried to seek async on a non-overlapped file handle"
            );

            Err(WinError::from(ERROR_INVALID_HANDLE))
        }
    }
}