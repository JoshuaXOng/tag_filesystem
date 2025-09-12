use std::{collections::BTreeSet, fmt::Display};

use fuser::FUSE_ROOT_ID;
use rand::random_range;

use crate::{errors::{AnyError, Result_}, unwrap_or, wrappers::write_btreeset};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct FileInode {
    id: u64,
}

impl FileInode {
    pub fn get_id(&self) -> u64 { self.id }

    pub fn try_from_free_inodes<T>(inodes_inuse: Vec<&T>)
    -> Result_<T> where T: TryFrom<u64> + PartialEq + Ord {
        loop {
            let inode_id = (random_range(
                NAMESPACE_INODE_END + 2..=(u64::MAX / 2)) * 2) - 1;
            let inode = unwrap_or!(T::try_from(inode_id), _e,
                continue);

            if !inodes_inuse.contains(&&inode) {
                return Ok(inode);
            }
        }
    }
}

impl TryFrom<u64> for FileInode {
    type Error = AnyError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !get_is_inode_a_file(value) {
            Err(format!("Not a valid file inode value `{value}`."))?;
        };
        Ok(Self {
            id: value,
        })
    }
}

impl Display for FileInode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct TagInode {
    id: u64,
}

impl TagInode {
    pub fn get_id(&self) -> u64 { self.id }

    pub fn try_from_free_inodes<T>(inodes_inuse: Vec<&T>)
    -> Result_<T> where T: TryFrom<u64> + PartialEq + Ord {
        loop {
            let inode_id = random_range(
                NAMESPACE_INODE_END + 1..=(u64::MAX / 2)) * 2;
            let inode = unwrap_or!(T::try_from(inode_id), _e,
                continue);

            if !inodes_inuse.contains(&&inode) {
                return Ok(inode);
            }
        }
    }
}

impl TryFrom<u64> for TagInode {
    type Error = AnyError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !get_is_inode_a_tag(value) {
            Err(format!("Not a valid tag inode value `{value}`."))?;
        };
        Ok(Self {
            id: value,
        })
    }
}

impl Display for TagInode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TagInodes(pub BTreeSet<TagInode>);

impl TagInodes {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
}

impl From<TagInode> for TagInodes {
    fn from(value: TagInode) -> Self {
        let mut tag_inodes = Self::new();
        tag_inodes.0.insert(value);
        tag_inodes
    }
}

impl Display for TagInodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_btreeset(f, &self.0)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct NamespaceInode {
    id: u64
}

impl NamespaceInode {
    pub fn get_id(&self) -> u64 { self.id }

    pub fn get_next(&self) -> Self {
        let is_at_end = self.id == NAMESPACE_INODE_END;
        if is_at_end {
            Self { id: NAMESPACE_INODE_START }
        } else {
            Self { id: self.id + 1 }
        }
    }
}

impl Display for NamespaceInode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl TryFrom<u64> for NamespaceInode {
    type Error = AnyError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !get_is_inode_a_namespace(value) {
            return Err(format!("Not a valid, in range id `{value}`.").into()); 
        }
        Ok(Self { id: value })
    }
}

pub fn get_is_inode_a_file(inode_id: u64) -> bool {
    inode_id > NAMESPACE_INODE_END && inode_id % 2 != 0
}

pub fn get_is_inode_a_tag(inode_id: u64) -> bool {
    inode_id > NAMESPACE_INODE_END && inode_id % 2 == 0
}

pub fn get_is_inode_a_namespace(inode_id: u64) -> bool {
    NAMESPACE_INODE_START <= inode_id && inode_id <= NAMESPACE_INODE_END
}

pub fn get_is_inode_root(inode_id: u64) -> bool {
    inode_id == FUSE_ROOT_ID
}

pub const NAMESPACE_INODE_START: u64 = FUSE_ROOT_ID + 1;
pub const ANY_NAMESPACE_INODE: u64 = NAMESPACE_INODE_START;

pub const NAMESPACE_INODE_END: u64 = 100;
