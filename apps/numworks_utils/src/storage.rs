use heapless::Vec;
use low_level_storage::{extapp_fileErase, extapp_fileExists, extapp_fileRead, extapp_fileWrite};

pub const MAX_STORAGE_VALUES: usize = 16;
// const MAX_FILE_SIZE: usize = MAX_VALUES * 4; // which represents 64 values (each represented by a u32)

pub mod low_level_storage {
    #[link(name = "storage")]
    extern "C" {
        #[allow(dead_code)]
        fn reverse32(value: u32) -> u32;
        #[allow(dead_code)]
        fn strcmp(s1: *const u8, s2: *const u8);
        pub fn extapp_fileExists(filename: *const u8) -> bool;
        pub fn extapp_fileRead(filename: *const u8, len: *mut u32) -> *const u8;
        pub fn extapp_fileWrite(filename: *const u8, content: *const u8, len: u32) -> bool;
        pub fn extapp_fileErase(filename: *const u8) -> bool;
        pub fn extapp_size() -> u32;
        pub fn extapp_address() -> u32;
        pub fn extapp_used() -> u32;
        pub fn extapp_nextFree() -> *mut u32;
        pub fn extapp_isValid(address: *mut u32) -> bool;
        pub fn extapp_calculatorModel() -> u8;
        pub fn extapp_userlandAddress() -> *mut u32;
    }
}

// Here are some helpers functions to read and write a file as a &[u32].

/// Takes the filename = name of the game, and opens the file associated : creates it if necessary, and returns True if it exists.
pub fn open_file(filename: &str) -> bool {
    if unsafe { extapp_fileExists(filename.as_ptr()) } {
        true
    } else {
        unsafe { extapp_fileWrite(filename.as_ptr(), core::ptr::null(), 0) }
    }
}

/// Like "extapp_fileRead" but returns a Vec<u32, MAX_VALUES> instead of a *const u8.
pub fn read_file(filename: &str) -> Vec<u32, MAX_STORAGE_VALUES> {
    if !unsafe { extapp_fileExists(filename.as_ptr()) } {
        Vec::new()
    } else {
        let mut len = 0;
        let data = unsafe { extapp_fileRead(filename.as_ptr(), &mut len) };
        let mut data_read = Vec::new();
        for i in (0..len as usize).step_by(4) {
            let v: u32 = unsafe { data.wrapping_add(i).cast::<u32>().read() };
            data_read.push(v).unwrap();
        }

        data_read
    }
}

pub fn read_data(filename: &str, pos: usize) -> Option<u32> {
    let values = read_file(filename);
    values.get(pos).copied()
}

/// Write a given data value to the file. Will copy the entirety of the file before erasing it and rewriting it again.
/// If a position is given, will replace this value. Else, writes it at the end of the file.
/// Returns the position at which it was added.
pub fn write_data(filename: &str, pos: Option<u32>, value: u32) -> usize {
    let mut values = read_file(filename);
    let res;
    if let Some(x) = pos {
        // if a position is given
        let d = values.get_mut(x as usize);
        if let Some(old_v) = d {
            // if data was found there, replace it
            *old_v = value;
            res = x as usize;
        } else {
            values.push(value).unwrap(); // else add it
            res = values.len() - 1;
        }
    } else {
        values.push(value).unwrap(); // else add it
        res = values.len() - 1;
    }
    unsafe { extapp_fileErase(filename.as_ptr()) };
    if !unsafe {
        extapp_fileWrite(
            filename.as_ptr(),
            values.as_ptr().cast(),
            values.len() as u32 * 4,
        )
    } {
        panic!("ERROR FILE WRITING\0");
    }
    res
}
