use std::{fmt::Display, time::SystemTime};

use fuser::FileType;

pub trait TfsEntry {
    fn get_name(&self) -> &str;
    fn get_inode_id(&self) -> u64;

    fn get_owner(&self) -> u32;
    fn get_group(&self) -> u32;
    fn get_permissions(&self) -> u16;

    fn get_file_kind(&self) -> FileType;

    fn get_when_accessed(&self) -> SystemTime;
    fn get_when_modified(&self) -> SystemTime;
    fn get_when_changed(&self) -> SystemTime;
    fn get_when_created(&self) -> SystemTime;
}

impl Display for dyn TfsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(id={})",
            self.get_name(),
            self.get_inode_id())
    }
}
