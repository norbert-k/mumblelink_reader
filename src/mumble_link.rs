use winapi::um::winnt::{HANDLE, PAGE_READWRITE};
use winapi::ctypes::{c_void, wchar_t};
use std::io;
use std::ffi::{OsStr, CString};
use std::os::windows::ffi::OsStrExt;
use winapi::um::handleapi::{INVALID_HANDLE_VALUE, CloseHandle};
use core::{ptr, mem};
use winapi::um::memoryapi::FILE_MAP_ALL_ACCESS;
use std::ptr::null;
use std::io::{ErrorKind, Error};
use winapi::shared::minwindef::FALSE;
use std::fmt;

fn wchar_t_to_string(src: &[wchar_t]) -> String {
    let zero = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..zero])
}

fn convert_to_imperial(position: &[f32; 3]) -> [f32; 3] {
    let mut imperial_position: [f32; 3] = [0.0, 0.0, 0.0];
    for (i, elem) in position.iter().enumerate() {
        imperial_position[i] = elem * 39.3701;
    }
    return imperial_position;
}

#[derive(Debug, Clone)]
pub struct MumbleLinkHandlerError {
    pub message: &'static str,
    pub os_error: bool,
}

impl fmt::Display for MumbleLinkHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Position {
    /// The character's position in space (in meters).
    pub position: [f32; 3],
    /// A unit vector pointing out of the character's eyes (in meters).
    pub front: [f32; 3],
    /// A unit vector pointing out of the top of the character's head (in meters).
    pub top: [f32; 3],
}

impl Position {
    #![feature(array_map)]
    pub fn to_imperial(&self) -> PositionImperial {
        PositionImperial {
            position: convert_to_imperial(&self.position),
            front: convert_to_imperial(&self.front),
            top: convert_to_imperial(&self.top),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct PositionImperial {
    /// The character's position in space (in inches).
    pub position: [f32; 3],
    /// A unit vector pointing out of the character's eyes (in inches).
    pub front: [f32; 3],
    /// A unit vector pointing out of the top of the character's head (in inches).
    pub top: [f32; 3],
}

#[derive(Copy, Debug)]
#[repr(C)]
struct MumbleLinkRawData {
    ui_version: u32,
    ui_tick: u32,
    avatar: Position,
    name: [wchar_t; 256],
    camera: Position,
    identity: [wchar_t; 256],
    context_len: u32,
    context: [u8; 256],
    description: [wchar_t; 2048],
}

impl MumbleLinkRawData {
    pub fn to_mumble_link_data(&self) -> MumbleLinkData {
        MumbleLinkData {
            ui_version: self.ui_version as i64,
            ui_tick: self.ui_tick as i64,
            avatar: self.avatar,
            name: wchar_t_to_string(&self.name),
            camera: self.camera,
            identity: wchar_t_to_string(&self.identity),
            context_len: self.context_len as i64,
            context: self.context,
            description: wchar_t_to_string(&self.description),
        }
    }
}

impl Clone for MumbleLinkRawData {
    fn clone(&self) -> Self { *self }
}

#[derive(Debug)]
pub struct MumbleLinkData {
    pub ui_version: i64,
    pub ui_tick: i64,
    pub avatar: Position,
    pub name: String,
    pub camera: Position,
    pub identity: String,
    pub context_len: i64,
    pub context: [u8; 256],
    pub description: String,
}

impl MumbleLinkData {
    pub fn read_context<T>(&self) -> T {
        let data: T = unsafe { std::ptr::read(self.context.as_ptr() as *const _) };
        return data;
    }
}

#[derive(Debug)]
#[cfg(all(windows))]
pub struct MumbleLinkHandler {
    handle: HANDLE,
    pub ptr: *mut c_void,
}

#[cfg(all(unix))]
pub struct MumbleLinkHandler {
    fd: libc::c_int,
    pub ptr: *mut c_void,
}

#[cfg(all(windows))]
lazy_static! {
    static ref wide_lp_name: Vec<u16> = OsStr::new("MumbleLink").encode_wide().chain(Some(0)).collect::<Vec<_>>();
}

#[cfg(all(unix))]
lazy_static! {
    static ref mmap_path: CString = unsafe {CString::new(format!("/MumbleLink.{}", libc::getpid())).unwrap() };
}

impl MumbleLinkHandler {
    #[cfg(all(windows))]
    pub fn new() -> io::Result<MumbleLinkHandler> {
        let mut handle: HANDLE = unsafe {
            winapi::um::memoryapi::CreateFileMappingW(INVALID_HANDLE_VALUE, ptr::null_mut(), PAGE_READWRITE, 0, std::mem::size_of::<MumbleLinkRawData>() as u32, wide_lp_name.as_ptr())
        };
        if handle.is_null() {
            handle = unsafe {
                winapi::um::memoryapi::OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, wide_lp_name.as_ptr())
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

    #[cfg(all(unix))]
    pub fn new() -> io::Result<MumbleLinkHandler> {
        unsafe {
            let fd = libc::shm_open(
                mmap_path.as_ptr(),
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

#[cfg(all(windows))]
impl Drop for MumbleLinkHandler {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
            self.ptr = ptr::null_mut()
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