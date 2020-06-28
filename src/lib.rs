#![feature(proc_macro_hygiene)]

use std::fs;
use skyline::{hook, install_hook, hooks::{Region, getRegionAddress}};
use skyline::libc::c_char;
use skyline::c_str;

mod resource;
mod replacement_files;

use resource::*;
use replacement_files::ARC_FILES;

static mut LOAD_LUA_FROM_INDEX: usize = 0x33a6130;
static mut LOAD_LUA_FILE: usize = 0x33a6e40;

#[skyline::from_offset(LOAD_LUA_FILE)]
fn load_lua_file(lua_state: *const LuaState, data: *const u8, size: u64, name: *const c_char) -> u64;

#[hook(offset = LOAD_LUA_FROM_INDEX)]
fn load_lua_from_index(lua_agent: &mut LuaAgent, index_ptr: &mut u32) -> u64 {
    let index = *index_ptr;

    let tables = LoadedTables::get_instance();
    let arc = tables.get_arc();
    let path_table = arc.file_info_path;
    let file_info = unsafe { &*path_table.offset(index as isize) };
    let hash = file_info.path.hash40.as_u64();

    if let Some(path) = ARC_FILES.0.get(&hash) {
        let contents = fs::read(&path).unwrap();
        let size = contents.len() as u64;

        let ret = unsafe { load_lua_file(lua_agent.lua_state, contents.as_ptr(), size, c_str("buf\0")) };

        if ret == 0 {
            0 // success
        } else {
            // ??? aaaaaaaaa some failure state
            unsafe {
                let lua_state = &mut lua_agent.lua_state;
                let mut top = lua_state.top;
                let func_next = (*lua_state.call_info).func.offset(1);
                while top < func_next {
                  lua_state.top = top.offset(1);
                  (*top).tt = 0;
                  top = lua_state.top;
                }
                lua_state.top = func_next;
            }

            0x40000004
        }
    } else {
        original!()(lua_agent, index_ptr)
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

static LOAD_LUA_FROM_INDEX_START_CODE: &[u8] = &[
      0xf9, 0x0f, 0x1b, 0xf8,     // str        x25,[sp, #local_50]!
      0xf8, 0x5f, 0x01, 0xa9,     // stp        x24,x23,[sp, #local_40]
      0xf6, 0x57, 0x02, 0xa9,     // stp        x22,x21,[sp, #local_30]
      0xf4, 0x4f, 0x03, 0xa9,     // stp        x20,x19,[sp, #local_20]
      0xfd, 0x7b, 0x04, 0xa9,     // stp        x29,x30,[sp, #local_10]
      0xfd, 0x03, 0x01, 0x91,     // add        x29,sp,#0x40
      0x08, 0x04, 0x40, 0xf9,     // ldr        x8,[lua_agent?, #0x8]
      0x93, 0x00, 0x80, 0x52,     // mov        w19,#0x4
      0x13, 0x00, 0xa8, 0x72,     // movk       w19,#0x4000, LSL #16
];

static LOAD_LUA_FILE_START_CODE: &[u8] = &[
      0xfc, 0x57, 0xbd, 0xa9,     // stp        x28,x21,[sp, #local_30]!
      0xf4, 0x4f, 0x01, 0xa9,     // stp        x20,x19,[sp, #local_20]
      0xfd, 0x7b, 0x02, 0xa9,     // stp        x29,x30,[sp, #local_10]
      0xfd, 0x83, 0x00, 0x91,     // add        x29,sp,#0x20
      0xff, 0x43, 0x10, 0xd1,     // sub        sp,sp,#0x410
      0x73, 0x00, 0x80, 0x52,     // mov        w19,#0x3
      0x13, 0x00, 0xa8, 0x72,     // movk       w19,#0x4000, LSL #16
      0x40, 0x01, 0x00, 0xb4,     // cbz        lua_state,LAB_71033a6e84
];

#[skyline::main(name = "replace_lua")]
pub fn main() {
    lazy_static::initialize(&ARC_FILES);

    unsafe {
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, LOAD_LUA_FILE_START_CODE) {
            LOAD_LUA_FILE = offset
        } else {
            println!("Error: no offset found. Defaulting to 7.0.0 offset. This likely won't work.");
        }
        
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, LOAD_LUA_FROM_INDEX_START_CODE) {
            LOAD_LUA_FROM_INDEX = offset
        } else {
            println!("Error: no offset found. Defaulting to 7.0.0 offset. This likely won't work.");
        }
    }

    install_hook!(load_lua_from_index);
}
