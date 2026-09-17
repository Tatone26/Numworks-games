use crate::storage::low_level_storage::{
    extapp_fileErase, extapp_fileExists, extapp_fileRead, extapp_fileWrite,
};
use crate::storage::MAX_STORAGE_VALUES;
use heapless::{String, Vec};

pub const MAX_SAVE_FIELDS: usize = MAX_STORAGE_VALUES;

/// Defines how many save slots a game supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveSlotMode {
    /// Exactly 1 save slot: uses `<app>.sav`. Overwriting or deleting targets this single file.
    Single,
    /// Up to `count` discrete slots: uses `<app>.s0`, `<app>.s1`, ...
    Multiple(u8),
}

#[derive(Clone, Copy, Debug)]
pub struct SaveField {
    pub name: &'static str,
    pub min: u32,
    pub max: u32,
    pub value: u32,
}

pub trait GameSave: Copy + Sized {
    /// Serializes game state into raw u32 words.
    fn to_words(&self) -> Vec<u32, MAX_STORAGE_VALUES>;

    /// Deserializes a slice of u32 words back into Self.
    fn from_words(words: &[u32]) -> Option<Self>;

    /// Exposes named fields for God Mode inspection and editing.
    fn describe_fields(&self) -> Vec<SaveField, MAX_SAVE_FIELDS>;

    /// Updates Self from modified field values.
    fn update_from_fields(&mut self, fields: &[SaveField]);
}

pub fn get_save_filename(app: &str, slot: Option<u8>) -> String<32> {
    use core::fmt::Write;
    let mut s = String::<32>::new();
    if let Some(slot_num) = slot {
        let _ = write!(s, "{}.s{}\0", app, slot_num);
    } else {
        let _ = write!(s, "{}.sav\0", app);
    }
    s
}

pub fn has_save(app: &str, slot: Option<u8>) -> bool {
    let filename = get_save_filename(app, slot);
    unsafe { extapp_fileExists(filename.as_ptr()) }
}

pub fn load_save<T: GameSave>(app: &str, slot: Option<u8>) -> Option<T> {
    let filename = get_save_filename(app, slot);
    if !unsafe { extapp_fileExists(filename.as_ptr()) } {
        return None;
    }

    let mut len: u32 = 0;
    let data_ptr = unsafe { extapp_fileRead(filename.as_ptr(), &mut len) };
    if data_ptr.is_null() || len < 4 {
        return None;
    }

    let mut words: Vec<u32, MAX_STORAGE_VALUES> = Vec::new();
    let word_count = (len as usize / 4).min(MAX_STORAGE_VALUES);

    for i in 0..word_count {
        let val: u32 = unsafe { data_ptr.add(i * 4).cast::<u32>().read() };
        words.push(val).ok()?;
    }

    T::from_words(&words)
}

pub fn write_save<T: GameSave>(app: &str, slot: Option<u8>, state: &T) -> bool {
    let filename = get_save_filename(app, slot);
    let words = state.to_words();

    if unsafe { extapp_fileExists(filename.as_ptr()) } {
        unsafe { extapp_fileErase(filename.as_ptr()) };
    }

    unsafe {
        extapp_fileWrite(
            filename.as_ptr(),
            words.as_ptr().cast(),
            words.len() as u32 * 4,
        )
    }
}

pub fn delete_save(app: &str, slot: Option<u8>) -> bool {
    let filename = get_save_filename(app, slot);
    if unsafe { extapp_fileExists(filename.as_ptr()) } {
        unsafe { extapp_fileErase(filename.as_ptr()) }
    } else {
        true
    }
}
