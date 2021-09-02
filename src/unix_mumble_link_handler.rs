use crate::mumble_link::{MumbleLinkReader, MumbleLinkData};
use crate::error::MumbleLinkHandlerError;
use std::io;
use core::ptr;
use std::ffi::CString;
use std::os::raw::c_uint;

#[cfg(all(unix))]
pub struct MumbleLinkHandler {
    fd: libc::c_int,
    pub ptr: *mut libc::c_void,
}

#[cfg(all(unix))]
lazy_static! {
    static ref MMAP_PATH: CString = CString::new(format!("/MumbleLink.{}", unsafe{libc::getuid()})).expect("Failed to create MMAP_PATH");
}

#[cfg(all(unix))]
impl MumbleLinkHandler {
    pub fn new() -> std::result::Result<MumbleLinkHandler, MumbleLinkHandlerError> {
        unsafe {
            let fd = libc::shm_open(
                MMAP_PATH.as_ptr(),
                libc::O_CREAT | libc::O_RDWR,
                libc::S_IRUSR | libc::S_IWUSR as c_uint,
            );
            if fd < 0 {
                println!("shm_open failed");
                return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
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
                println!("mmap failed");
                libc::close(fd);
                return Err(MumbleLinkHandlerError::OSError(io::Error::last_os_error()));
            }
            Ok(MumbleLinkHandler {
                fd: fd,
                ptr: ptr,
            })
        }
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