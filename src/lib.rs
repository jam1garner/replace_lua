#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook, hooks::{Region, getRegionAddress}};
use std::sync::atomic::AtomicU32;
use skyline::logging::HexDump;

fn offset_to_addr(offset: usize) -> *const () {
    unsafe {
        (getRegionAddress(Region::Text) as *const u8).offset(offset as isize) as _
    }
}

struct LuaState {

}

#[repr(packed)]
struct Table1Entry {
    table2_index: u32,
    is_in_table_2: u32,
}

#[repr(C)]
#[derive(Debug)]
struct Table2Entry {
    data: *const u8,
    ref_count: AtomicU32,
    is_used: bool,
    unk2: [u8; 0xb]
}

struct LuaAgent<'a> {
    unk: u64,
    lua_state: &'a LuaState
}

struct CppVector<T> {
    start: *const T,
    end: *const T,
    eos: *const T
}

#[repr(C)]
struct LoadedTables {
    mutex: *const (),
    table1: *const Table1Entry,
    table2: *const Table2Entry,
    table1_len: u32,
    table2_len: u32,
    table1_count: u32,
    unk: u32,
    table1_list: CppVector<u32>,
    loaded_directory_table: *const (),
    loaded_directory_table_size: u32,
    unk2: u32,
    unk3: CppVector<u32>,
    unk4: u8,
    unk5: [u8; 7],
    addr: *const (),
    loaded_data: &'static mut LoadedData,
    version: u32,
}

struct LoadedData {
    arc: &'static mut LoadedArc
}

struct LoadedArc {
    magic: u64,
    music_data_offset: u64,
    file_data_offset: u64,
    file_data_offset_2: u64,
    fs_offset: u64,
    fs_search_offset: u64,
    unk_offset: u64,
    region_entry: *const (),
    file_path_buckets: *const (),
    file_path_to_index_hash_group: *const (),
    file_info_path: *const FileInfoPath
}

struct FileInfoPath {
    path: HashIndexGroup,
    extension: HashIndexGroup,
    parent: HashIndexGroup,
    file_name: HashIndexGroup,
}

#[repr(packed)]
struct HashIndexGroup {
    hash40: Hash40,
    flags: [u8; 3]
}

#[repr(packed)]
#[derive(Copy, Clone)]
struct Hash40 {
    crc32: u32,
    len: u8
}

impl Hash40 {
    pub fn as_u64(&self) -> u64 {
        (self.crc32 as u64) + ((self.len as u64) << 32)
    }

    pub fn from_u64(hash40: u64) -> Self {
        Self {
            crc32: hash40 as u32,
            len: (hash40 >> 32) as u8
        }
    }
}

impl LoadedTables {
    fn get_arc(&mut self) -> &mut LoadedArc {
        self.loaded_data.arc
    }

    fn table_1(&self) -> &[Table1Entry] {
        unsafe {
            std::slice::from_raw_parts(self.table1, self.table1_len as usize)
        }
    }
    
    fn table_2(&self) -> &[Table2Entry] {
        unsafe {
            std::slice::from_raw_parts(self.table2, self.table2_len as usize)
        }
    }

    fn get_instance() -> &'static mut Self {
        unsafe {
            let x: *mut &'static mut Self= std::mem::transmute(offset_to_addr(0x4e05490));
            *x
        }
    }
}

#[hook(offset = 0x33a6130)]
fn load_lua_from_index(lua_agent: &mut LuaAgent, index_ptr: &mut u32) -> u64 {
    let index = unsafe { *index_ptr };
    // println!("index: {}", index);
    // println!("tables: {:?}", &LoadedTables::get_instance() as *const _);
    // println!("loaded_data: {:?}", LoadedTables::get_instance().loaded_data as *const _);
    // println!("{}", HexDump(LoadedTables::get_instance()));
    //skyline::logging::hex_dump_ptr(&LoadedTables::get_instance() as *const _);

    let tables = LoadedTables::get_instance();
    let arc = tables.get_arc();
    let path_table = arc.file_info_path;

    unsafe {
        let file_info = &*path_table.offset(index as isize);
        println!("Hash40 = 0x{:x}", file_info.path.hash40.as_u64());
    }
    
    original!()(lua_agent, index_ptr)
}

#[skyline::main(name = "replace_lua")]
pub fn main() {
    install_hook!(load_lua_from_index);
}
