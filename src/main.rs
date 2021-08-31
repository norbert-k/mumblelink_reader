use crate::mumble_link::MumbleLinkHandler;
use std::thread;
use core::time;

mod mumble_link;

#[macro_use]
extern crate lazy_static;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct GuildwarsContext {
    pub server_address: [u8; 28],
    pub map_id: u32,
    pub map_type: u32,
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    pub ui_state: u32,
    pub compass_width: u16,
    pub compass_height: u16,
    pub compass_rotation: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub map_center_x: f32,
    pub map_center_y: f32,
    pub map_scale: f32,
    pub process_id: u32,
    pub mount_index: u8,
}

fn main() {
    let handler = MumbleLinkHandler::new().unwrap();
    loop {
        let linked_memory = handler.read().unwrap();
        println!("{:?}", linked_memory.read_context::<GuildwarsContext>());
        thread::sleep(time::Duration::from_millis(5000))
    }
}