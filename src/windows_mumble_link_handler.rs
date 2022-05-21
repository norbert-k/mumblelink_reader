use core::ptr;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::FILE_MAP_ALL_ACCESS;
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use crate::error::MumbleLinkHandlerError;
use crate::mumble_link::CMumbleLinkData;

#[derive(Debug)]
#[cfg(all(windows))]
/// MumbleLink entry point, opens or initializes *Memory Mapped* file to read MumbleLink data from.
pub struct MumbleLinkHandler {
    /// Handle to the specified file mapping object.
    handle: HANDLE,
    /// Starting address of the mapped view.
    pub(crate) ptr: *mut c_void,
}

#[cfg(all(windows))]
lazy_static! {
    static ref WIDE_LP_NAME: Vec<u16> = OsStr::new("MumbleLink")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    static ref MUMBLE_LINK_STRUCT_SIZE: usize = std::mem::size_of::<CMumbleLinkData>() as usize;
}

#[cfg(all(windows))]
impl MumbleLinkHandler {
    fn create(path: &[u16]) -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        // Try to open MumbleLink with OpenFileMappingW, if it fails to acquire handle then execute CreateFileMappingW to create it.
        let mut handle: HANDLE = unsafe {
            winapi::um::memoryapi::OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, path.as_ptr())
        };
        if handle.is_null() {
            handle = unsafe {
                winapi::um::memoryapi::CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    ptr::null_mut(),
                    PAGE_READWRITE,
                    0,
                    *MUMBLE_LINK_STRUCT_SIZE as u32,
                    WIDE_LP_NAME.as_ptr(),
                )
            };
            if handle.is_null() {
                return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
            }
        }
        let ptr: *mut c_void = unsafe {
            winapi::um::memoryapi::MapViewOfFile(
                handle,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                *MUMBLE_LINK_STRUCT_SIZE,
            ) as *mut c_void
        };
        if ptr.is_null() {
            unsafe {
                CloseHandle(handle);
            }
            return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
        }
        Ok(MumbleLinkHandler { handle, ptr })
    }

    /// Create new MumbleLinkHandler
    pub fn new() -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        Self::create(WIDE_LP_NAME.as_slice())
    }

    /// Create new MumbleLinkHandler with specified name
    pub fn with_name(name: &str) -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        let name = OsStr::new(name)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        Self::create(name.as_slice())
    }
}

#[cfg(all(windows))]
impl Drop for MumbleLinkHandler {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
            self.ptr = ptr::null_mut()
        }
    }
}
