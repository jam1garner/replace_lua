#![feature(proc_macro_hygiene)]

use std::{fmt, fs};
use skyline::{hook, install_hook, hooks::{Region, getRegionAddress}};
use skyline::libc::{c_char, free};
use skyline::{from_c_str, c_str};
use smash::lib::{L2CValue, L2CAgent, L2CValueType};
use skyline::logging::{HexDump, hex_dump_ptr};

mod resource;
mod replacement_files;

use resource::*;
use replacement_files::ARC_FILES;

static mut LOAD_LUA_FROM_INDEX: usize = 0x33a6130;
static mut LOAD_LUA_FILE: usize = 0x33a6e40;

// 7.0.0
//static mut LUA_PRINT: usize = 0x3579310;

// 8.0.0
static mut LUA_PRINT: usize = 0x360cf90;

// static mut LUA_PUSH_FORMAT_STRING: usize = 0x357fd60;
// static mut ERROR_FORMAT_STRING_OFFSET: usize = 0x3e81f93;

#[skyline::from_offset(LOAD_LUA_FILE)]
fn load_lua_file(lua_state: *const LuaState, data: *const u8, size: u64, name: *const c_char, some_flag: u64) -> u64;

struct LuaDisp(L2CValue);

impl fmt::Display for LuaDisp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.val_type {
            L2CValueType::Void => write!(f, "nil"),
            L2CValueType::Bool => write!(f, "{}", unsafe { self.0.inner.raw } & 1 != 0),
            L2CValueType::Int => write!(f, "{}", unsafe { self.0.inner.raw }),
            L2CValueType::Num => write!(f, "{}", unsafe { self.0.inner.raw_float }),
            L2CValueType::Pointer => write!(f, "ptr(0x{:x})", unsafe { self.0.inner.raw }),
            L2CValueType::String =>
                write!(f, "{}", unsafe { &from_c_str((self.0.inner.raw_pointer as *const _))[1..] }),
            _ => write!(f, "{:?}", self.0)
        }
    }
}

#[hook(offset = LUA_PRINT)]
fn luaB_print(lua_state: &mut LuaState) -> u32 {
    let mut lua_agent = L2CAgent::new(lua_state as *mut _ as u64);

    unsafe {
        let val = lua_agent.pop_lua_stack(1);
        println!("[lua] {}", LuaDisp(val));
    }

    0
}

#[hook(offset = LOAD_LUA_FROM_INDEX)]
fn load_lua_from_index(lua_agent: *mut LuaAgent, index_ptr: *mut u32, some_flag: u64) -> u64 {
    if index_ptr.is_null() {
        return 0x40000004
    }
    
    if lua_agent.is_null() {
        return 0x40000003
    }

    let index = unsafe { *index_ptr };

    let tables = LoadedTables::get_instance();
    let hash = tables.get_hash_from_t1_index(index).as_u64();
    if let Some(path) = ARC_FILES.0.get(&hash) {
        let contents = fs::read(&path).unwrap();
        let size = contents.len() as u64;

        let ret = unsafe { load_lua_file((*lua_agent).lua_state, contents.as_ptr(), size, c_str("buf\0"), some_flag & 1) };

        if ret == 0 {
            0 // success
        } else {
            // ??? aaaaaaaaa some failure state
            unsafe {
                let lua_state = &mut (*lua_agent).lua_state;
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
        original!()(lua_agent, index_ptr, some_flag)
    }
}

/*#[hook(offset = 0x31be530)]
fn load_sync_by_file_path_id(loaded_tables: &mut LoadedTables, index: u32) {
    original!()(loaded_tables, index);
    let hash = loaded_tables.get_hash_from_t1_index(index).as_u64();
    if let Some(path) = ARC_FILES.0.get(&hash) {
        println!("Loading sync: {:?}", path);
        let t2_entry = loaded_tables.get_t2_mut(index).unwrap();
        if t2_entry.data.is_null() {
            println!("Is null.");
            return;
        }
        hex_dump_ptr(t2_entry.data);
        println!("a");
        println!("b");
        println!("c");
        unsafe {
            free(t2_entry.data as *const _);
        }
        let data = fs::read(path).unwrap().into_boxed_slice();
        let data = Box::leak(data);
        println!("{}", HexDump(t2_entry));
        hex_dump_ptr(data.as_ptr());
        t2_entry.data = data.as_ptr();
    }
}*/

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

static LOAD_LUA_FROM_INDEX_START_CODE: &[u8] = &[
      0xfa, 0x67, 0xbb, 0xa9,     // stp        x26,x25,[sp, #local_50]!
      0xf8, 0x5f, 0x01, 0xa9,     // stp        x24,x23,[sp, #local_40]
      0xf6, 0x57, 0x02, 0xa9,     // stp        x22,x21,[sp, #local_30]
      0xf4, 0x4f, 0x03, 0xa9,     // stp        x20,x19,[sp, #local_20]
      0xfd, 0x7b, 0x04, 0xa9,     // stp        x29,x30,[sp, #local_10]
      0xfd, 0x03, 0x01, 0x91,     // add        x29,sp,#0x40
      0x08, 0x04, 0x40, 0xf9,     // ldr        x8,[x0, #param_1->common_header.tt]
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
];

#[skyline::main(name = "replace_lua")]
pub fn main() {
    lazy_static::initialize(&ARC_FILES);

    unsafe {
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, LOAD_LUA_FILE_START_CODE) {
            println!("[replace_lua] load_lua_file found: 0x{:x}.", offset);
            LOAD_LUA_FILE = offset
        } else {
            println!("Error: no offset found. Defaulting to 7.0.0 offset. This likely won't work.");
        }
        
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, LOAD_LUA_FROM_INDEX_START_CODE) {
            println!("[replace_lua] load_lua_from_index found: 0x{:x}.", offset);
            LOAD_LUA_FROM_INDEX = offset
        } else {
            println!("Error: no offset found. Defaulting to 7.0.0 offset. This likely won't work.");
        }
    }

    install_hook!(load_lua_from_index);
    install_hook!(luaB_print);
    //install_hook!(load_sync_by_file_path_id);
    //install_hook!(lua_push_format_string);
}
