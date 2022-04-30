use crate::error::MumbleLinkHandlerError;
use crate::mumble_link::{CMumbleLinkData, MumbleLinkData, MumbleLinkReader};
use core::ptr;
use std::ffi::CString;
use std::io;
use std::os::raw::c_uint;

#[cfg(all(unix))]
pub struct MumbleLinkHandler {
    fd: libc::c_int,
    pub ptr: *mut libc::c_void,
}

#[cfg(all(unix))]
impl MumbleLinkHandler {
    /// Create new MumbleLinkHandler with specified name
    pub fn with_name(name: &str) -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        let uid = unsafe { libc::getuid() };
        let MUMBLE_LINK_STRUCT_SIZE: i64 = std::mem::size_of::<CMumbleLinkData>() as i64;
        let MMAP_PATH: CString =
            CString::new(format!("/{}.{}", name, uid)).expect("Failed to create MMAP_PATH");
        unsafe {
            let mut fd = libc::shm_open(
                MMAP_PATH.as_ptr(),
                libc::O_RDWR,
                libc::S_IRUSR as c_uint | libc::S_IWUSR as c_uint,
            );
            if fd < 0 {
                fd = libc::shm_open(
                    MMAP_PATH.as_ptr(),
                    libc::O_CREAT,
                    libc::S_IRUSR as c_uint | libc::S_IWUSR as c_uint,
                );
                let trunc: i32 = unsafe { libc::ftruncate(fd, MUMBLE_LINK_STRUCT_SIZE) };
                if trunc < 0 || fd < 0 {
                    return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
                }
            }
            let ptr = libc::mmap(
                ptr::null_mut(),
                std::mem::size_of::<MumbleLinkData>(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            if ptr as isize == -1 {
                libc::close(fd);
                return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
            }
            Ok(MumbleLinkHandler { fd: fd, ptr: ptr })
        }
    }

    /// Create new MumbleLinkHandler
    pub fn new() -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        Self::with_name("MumbleLink")
    }
}

#[cfg(all(unix))]
impl Drop for MumbleLinkHandler {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
            self.ptr = ptr::null_mut()
        }
    }
}
