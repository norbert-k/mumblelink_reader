//! Mumblelink reader
//!
//! Provides an abstraction over Mumblelink for Windows and Unix like systems

pub mod mumble_link;
pub mod error;
#[cfg_attr(windows, path="windows_mumble_link_handler.rs")]
#[cfg_attr(unix, path="unix_mumble_link_handler.rs")]
pub mod mumble_link_handler;

#[macro_use]
extern crate lazy_static;