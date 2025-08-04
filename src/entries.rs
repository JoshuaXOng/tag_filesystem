use std::ffi::OsString;

use fuser::FileAttr;

pub trait TfsEntry {
    fn get_name(&self) -> OsString;
    fn get_raw_inode(&self) -> u64;
    fn get_attributes(&self) -> FileAttr;
}
