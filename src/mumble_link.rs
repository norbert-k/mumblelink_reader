use core::ptr;
use std::ffi::OsStr;
use std::fmt;
use std::io;

#[cfg(all(windows))]
use winapi::{
    shared::minwindef::FALSE,
    um::{
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        memoryapi::FILE_MAP_ALL_ACCESS,
        winnt::{HANDLE, PAGE_READWRITE},
    },
};

use crate::error::MumbleLinkHandlerError;
use crate::mumble_link_handler::MumbleLinkHandler;
use libc::wchar_t;

#[cfg(all(windows))]
fn wchar_t_to_string(src: &[wchar_t]) -> String {
    let zero = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..zero])
}

#[cfg(all(unix))]
fn wchar_t_to_string(src: &[wchar_t]) -> String {
    let zero = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf8(src[..zero].to_vec().iter_mut().map(|x| *x as u8).collect()).unwrap()
}

fn convert_to_imperial(position: &[f32; 3]) -> [f32; 3] {
    let mut imperial_position: [f32; 3] = [0.0, 0.0, 0.0];
    for (i, elem) in position.iter().enumerate() {
        imperial_position[i] = elem * 39.3701;
    }
    imperial_position
}

/// Three dimensional Vector
pub type Vector3D = [f32; 3];

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// Character position in Left hand coordinate system (in SI base units: meters)
pub struct Position {
    /// The character's position in space (in meters).
    pub position: Vector3D,
    /// A unit vector pointing out of the character's eyes (in meters).
    pub front: Vector3D,
    /// A unit vector pointing out of the top of the character's head (in meters).
    pub top: Vector3D,
}

impl Position {
    pub fn to_imperial(self) -> PositionImperial {
        PositionImperial {
            position: convert_to_imperial(&self.position),
            front: convert_to_imperial(&self.front),
            top: convert_to_imperial(&self.top),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// Character position in Left hand coordinate system (in imperial units: inches)
pub struct PositionImperial {
    /// The character's position in space (in inches).
    pub position: Vector3D,
    /// A unit vector pointing out of the character's eyes (in inches).
    pub front: Vector3D,
    /// A unit vector pointing out of the top of the character's head (in inches).
    pub top: Vector3D,
}

#[derive(Copy, Debug)]
#[repr(C)]
/// MumbleLink data in repr(C) format
pub struct CMumbleLinkData {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub avatar: Position,
    pub name: [wchar_t; 256],
    pub camera: Position,
    pub identity: [wchar_t; 256],
    pub context_len: u32,
    pub context: [u8; 256],
    pub description: [wchar_t; 2048],
}

impl CMumbleLinkData {
    pub fn to_mumble_link_data(self) -> MumbleLinkData {
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

impl Clone for CMumbleLinkData {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Clone, Debug)]
/// MumbleLink data
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

/// MumbleLink context reading
pub trait MumbleLinkDataReader {
    /// Read MumbleLinkData context into *T* struct from `[u8; 256]` representation
    fn read_context_into_struct<T>(&self) -> T;
    /// Transform MumbleLinkData context into T from `[u8; 256]` representation
    fn read_context<T>(&self, f: &dyn Fn([u8; 256]) -> T) -> T;
}

impl MumbleLinkDataReader for MumbleLinkData {
    /// Read MumbleLinkData context into *T* struct from `[u8; 256]` representation
    fn read_context_into_struct<T>(&self) -> T {
        unsafe { std::ptr::read(self.context.as_ptr() as *const _) }
    }

    /// Transform MumbleLinkData context into T from `[u8; 256]` representation
    fn read_context<T>(&self, f: &dyn Fn([u8; 256]) -> T) -> T {
        f(self.context)
    }
}

impl MumbleLinkDataReader for CMumbleLinkData {
    /// Read MumbleLinkData context into *T* struct from `[u8; 256]` representation
    fn read_context_into_struct<T>(&self) -> T {
        unsafe { std::ptr::read(self.context.as_ptr() as *const _) }
    }

    /// Transform MumbleLinkData context into T from `[u8; 256]` representation
    fn read_context<T>(&self, f: &dyn Fn([u8; 256]) -> T) -> T {
        f(self.context)
    }
}

/// MumbleLink data reader
pub trait MumbleLinkReader {
    /// Read into regular struct: *MumbleLinkData*
    fn read(&self) -> std::result::Result<MumbleLinkData, MumbleLinkHandlerError>;
    /// Read into repr(C) struct: *CMumbleLinkData*
    fn read_c(&self) -> std::result::Result<CMumbleLinkData, MumbleLinkHandlerError>;
}

impl MumbleLinkReader for MumbleLinkHandler {
    /// Read into regular struct: *MumbleLinkData*
    fn read(&self) -> Result<MumbleLinkData, MumbleLinkHandlerError> {
        if self.ptr.is_null() {
            return Err(MumbleLinkHandlerError::UnableToRead);
        }
        let linked_memory = unsafe { ptr::read_unaligned(self.ptr as *mut CMumbleLinkData) };
        Ok(linked_memory.to_mumble_link_data())
    }

    /// Read into repr(C) struct: *CMumbleLinkData*
    fn read_c(&self) -> Result<CMumbleLinkData, MumbleLinkHandlerError> {
        if self.ptr.is_null() {
            return Err(MumbleLinkHandlerError::UnableToRead);
        }
        Ok(unsafe { ptr::read_unaligned(self.ptr as *mut CMumbleLinkData) })
    }
}
