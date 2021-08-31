use core::ptr;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;

use winapi::ctypes::{c_void};
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::FILE_MAP_ALL_ACCESS;
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use crate::mumble_link::{MumbleLinkData, MumbleLinkRawData};
use crate::error::MumbleLinkHandlerError;

#[derive(Debug)]
#[cfg(all(windows))]
pub struct MumbleLinkHandler {
    handle: HANDLE,
    pub ptr: *mut c_void,
}

#[cfg(all(windows))]
lazy_static! {
    static ref WIDE_LP_NAME: Vec<u16> = OsStr::new("MumbleLink").encode_wide().chain(Some(0)).collect::<Vec<_>>();
}

#[cfg(all(windows))]
impl MumbleLinkHandler {
    pub fn new() -> io::Result<MumbleLinkHandler> {
        let mut handle: HANDLE = unsafe {
            winapi::um::memoryapi::OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, WIDE_LP_NAME.as_ptr())
        };
        if handle.is_null() {
            handle = unsafe {
                winapi::um::memoryapi::CreateFileMappingW(INVALID_HANDLE_VALUE, ptr::null_mut(), PAGE_READWRITE, 0, std::mem::size_of::<MumbleLinkRawData>() as u32, WIDE_LP_NAME.as_ptr())
            };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
        }
        let ptr: *mut c_void = unsafe {
            winapi::um::memoryapi::MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, std::mem::size_of::<MumbleLinkRawData>()) as *mut c_void
        };
        if ptr.is_null() {
            unsafe { CloseHandle(handle); }
            return Err(io::Error::last_os_error());
        }
        Ok(MumbleLinkHandler {
            handle,
            ptr,
        })
    }

    pub fn read(&self) -> std::result::Result<MumbleLinkData, MumbleLinkHandlerError> {
        if self.ptr.is_null() {
            return Err(MumbleLinkHandlerError {
                message: "Failed to read MumbleLink data",
                os_error: false,
            });
        }
        let linked_memory = unsafe { ptr::read_unaligned(self.ptr as *mut MumbleLinkRawData) };
        Ok(linked_memory.to_mumble_link_data())
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