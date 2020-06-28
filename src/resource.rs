use std::fmt;
use skyline::hooks::{Region, getRegionAddress};
use std::sync::atomic::AtomicU32;

fn offset_to_addr(offset: usize) -> *const () {
    unsafe {
        (getRegionAddress(Region::Text) as *const u8).offset(offset as isize) as _
    }
}

#[repr(C)]
pub struct LuaState {
    pub ignore: [u8; 0x10],
    pub top: *mut TValue,
    pub global_state: *const (),
    pub call_info: *mut CallInfo,
}

#[repr(C)]
pub struct TValue {
    pub val: u64,
    pub tt: u32
}

#[repr(C)]
pub struct CallInfo {
    pub func: *mut TValue,
    pub top: *mut TValue,
    pub prev: *mut CallInfo,
    pub next: *mut CallInfo,
    // more...
}

#[repr(C)]
#[repr(packed)]
pub struct Table1Entry {
    pub table2_index: u32,
    pub is_in_table_2: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Table2Entry {
    pub data: *const u8,
    pub ref_count: AtomicU32,
    pub is_used: bool,
    pub unk2: [u8; 0xb]
}

#[repr(C)]
pub struct LuaAgent<'a> {
    pub unk: u64,
    pub lua_state: &'a mut LuaState
}

#[repr(C)]
pub struct CppVector<T> {
    start: *const T,
    end: *const T,
    eos: *const T
}

#[repr(C)]
pub struct LoadedTables {
    pub mutex: *const (),
    pub table1: *const Table1Entry,
    pub table2: *const Table2Entry,
    pub table1_len: u32,
    pub table2_len: u32,
    pub table1_count: u32,
    pub unk: u32,
    pub table1_list: CppVector<u32>,
    pub loaded_directory_table: *const (),
    pub loaded_directory_table_size: u32,
    pub unk2: u32,
    pub unk3: CppVector<u32>,
    pub unk4: u8,
    pub unk5: [u8; 7],
    pub addr: *const (),
    pub loaded_data: &'static mut LoadedData,
    pub version: u32,
}

#[repr(C)]
pub struct LoadedData {
    arc: &'static mut LoadedArc
}

#[repr(C)]
pub struct LoadedArc {
    pub magic: u64,
    pub music_data_offset: u64,
    pub file_data_offset: u64,
    pub file_data_offset_2: u64,
    pub fs_offset: u64,
    pub fs_search_offset: u64,
    pub unk_offset: u64,
    pub loaded_fs: *const (),
    pub loaded_fs_2: *const (),
    pub region_entry: *const (),
    pub file_path_buckets: *const (),
    pub file_path_to_index_hash_group: *const (),
    pub file_info_path: *const FileInfoPath,
    pub file_info_idx: *const (),
    pub dir_hash_group: *const (),
    pub dir_list: *const (),
    pub dir_offset: *const (),
    pub dir_child_hash_group: *const (),
    pub file_info: *const FileInfo,
}

#[repr(C)]
pub struct FileInfo {
    pub path_index: u32,
    pub index_index: u32,
    pub sub_index_index: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct FileInfoPath {
    pub path: HashIndexGroup,
    pub extension: HashIndexGroup,
    pub parent: HashIndexGroup,
    pub file_name: HashIndexGroup,
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct HashIndexGroup {
    pub hash40: Hash40,
    pub flags: [u8; 3]
}

impl fmt::Debug for HashIndexGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.hash40.as_u64())
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Hash40 {
    crc32: u32,
    len: u8
}

impl Hash40 {
    pub fn as_u64(&self) -> u64 {
        (self.crc32 as u64) + ((self.len as u64) << 32)
    }
}

impl LoadedTables {
    pub fn get_arc(&mut self) -> &mut LoadedArc {
        self.loaded_data.arc
    }

    pub fn get_instance() -> &'static mut Self {
        unsafe {
            let instance_ptr: *mut &'static mut Self= std::mem::transmute(offset_to_addr(0x4e05490));
            *instance_ptr
        }
    }
}
