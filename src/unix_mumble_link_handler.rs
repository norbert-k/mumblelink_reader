#[cfg(all(unix))]
pub struct MumbleLinkHandler {
    fd: libc::c_int,
    pub ptr: *mut c_void,
}

#[cfg(all(unix))]
lazy_static! {
    static ref MMAP_PATH: CString = unsafe {CString::new(format!("/MumbleLink.{}", libc::getpid())).unwrap() };
}

#[cfg(all(unix))]
impl MumbleLinkHandler {
    pub fn new() -> io::Result<MumbleLinkHandler> {
        unsafe {
            let fd = libc::shm_open(
                MMAP_PATH.as_ptr(),
                libc::O_RDWR,
                libc::S_IRUSR | libc::S_IWUSR,
            );
            if fd < 0 {
                return Err(io::Error::last_os_error());
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
                return Err(io::Error::last_os_error());
            }
            Ok(MumbleLinkHandler {
                fd: fd,
                ptr: ptr,
            })
        }
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

#[cfg(all(unix))]
impl Drop for MumbleLinkHandler {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
            self.ptr = ptr::null_mut()
        }
    }
}