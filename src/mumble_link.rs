use core::ptr;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::os::windows::ffi::OsStrExt;

use winapi::ctypes::{c_void, wchar_t};
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::FILE_MAP_ALL_ACCESS;
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use crate::error::MumbleLinkHandlerError;
use crate::mumble_link_handler::MumbleLinkHandler;

fn wchar_t_to_string(src: &[wchar_t]) -> String {
    let zero = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..zero])
}

fn convert_to_imperial(position: &[f32; 3]) -> [f32; 3] {
    let mut imperial_position: [f32; 3] = [0.0, 0.0, 0.0];
    for (i, elem) in position.iter().enumerate() {
        imperial_position[i] = elem * 39.3701;
    }
    imperial_position
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
pub struct MumbleLinkRawData {
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
    pub fn read_context_into_struct<T>(&self) -> T {
        unsafe { std::ptr::read(self.context.as_ptr() as *const _) }
    }

    pub fn read_context<T>(&self, f: &dyn Fn([u8; 256]) -> T) -> T {
        f(self.context)
    }
}

pub trait MumbleLinkReader {
    fn read(&self) -> std::result::Result<MumbleLinkData, MumbleLinkHandlerError>;
}

impl MumbleLinkReader for MumbleLinkHandler {
    fn read(&self) -> Result<MumbleLinkData, MumbleLinkHandlerError> {
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