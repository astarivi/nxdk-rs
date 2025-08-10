use crate::io::traits::{Read, Seek, SeekFrom, Write};
use crate::winapi::error::{NtStatusError, WinError, WinMixedError};
use crate::winapi::WindowsPath;
use core::ffi::c_void;
use bitflags::bitflags;
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
pub struct WinHandle {
    handle: Option<HANDLE>
}

unsafe impl Send for WinHandle {}

impl WinHandle {
    /// Be careful to never initialize this with a search HANDLE. Doing so
    /// will result in unexpected panics.
    pub fn new(handle: HANDLE) -> Self {
        WinHandle{
            handle: Some(handle)
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
                handle: Some(handle)
            }
        )
    }

    fn get_inner(&self) -> Result<HANDLE, WinError> {
        self.handle.ok_or(WinError::from(ERROR_INVALID_HANDLE))
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
        !self.handle.is_some()
    }

    pub fn close(&mut self) -> Result<(), WinError> {
        let result = unsafe {
            CloseHandle(
                self.get_inner()?
            )
        };

        if result == 0 {
            return Err(WinError::from_last_error())
        }

        self.handle = None;

        Ok(())
    }
}

impl Write for WinHandle {
    type WriteError = WinError;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::WriteError> {
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

    fn flush(&mut self) -> Result<(), Self::WriteError> {
        panic!("Flushing WinNT file handles causes a deadlock")
    }
}

impl Read for WinHandle {
    type ReadError = WinError;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::ReadError> {
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

impl Seek for WinHandle {
    type SeekError = WinError;

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::SeekError> {
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