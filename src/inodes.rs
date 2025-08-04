use std::{collections::BTreeSet, fmt::Display, marker::PhantomData};

use fuser::FUSE_ROOT_ID;

use crate::{errors::{AnyError, Result_}, unwrap_or};

// TODO: Implement Display for deez.
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct FileInode {
    id: u64,
    _pd: PhantomData<()>
}

impl FileInode {
    pub fn get_id(&self) -> u64 { self.id }
}

impl TryFrom<u64> for FileInode {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !is_inode_a_file(value) {
            Err(format!("Not a valid file inode value `{value}`."))?;
        };
        Ok(Self {
            id: value,
            _pd: PhantomData,
        })
    }
}

pub type TagInodes = BTreeSet<TagInode>;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct TagInode {
    id: u64,
    _pd: PhantomData<()>
}

impl TagInode {
    pub fn get_id(&self) -> u64 { self.id }
}

impl TryFrom<u64> for TagInode {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !is_inode_a_tag(value) {
            Err(format!("Not a valid tag inode value `{value}`."))?;
        };
        Ok(Self {
            id: value,
            _pd: PhantomData,
        })
    }
}

pub fn try_from_free_inode<T>(inodes_inuse: Vec<&T>)
-> Result_<T>
where T: TryFrom<u64> + PartialEq {
    for inode_id in QUERY_INODE_END + 1.. {
        let inode = unwrap_or!(T::try_from(inode_id), _e, {
            continue;
        });

        let is_used = inodes_inuse.contains(&&inode);
        if !is_used {
            return Ok(inode);
        }
    }
    Err("Ran out of space for inodes.".into())
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct QueryInode {
    id: u64
}

impl QueryInode {
    pub fn get_id(&self) -> u64 { self.id }

    pub fn get_next(&self) -> Self {
        if self.id == QUERY_INODE_END {
            Self { id: QUERY_INODE_START }
        } else {
            Self { id: self.id + 1 }
        }
    }
}

impl TryFrom<u64> for QueryInode {
    // TODO: Change others to AnyError
    type Error = AnyError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !is_inode_a_query(value) {
            return Err(format!("Not a valid, in range id `{value}`.").into()); 
        }
        Ok(Self { id: value })
    }
}

pub fn is_inode_a_file(inode_id: u64) -> bool {
    inode_id > QUERY_INODE_END && inode_id % 2 != 0
}

pub fn is_inode_a_tag(inode_id: u64) -> bool {
    inode_id > QUERY_INODE_END && inode_id % 2 == 0
}

pub fn is_inode_a_query(inode_id: u64) -> bool {
    QUERY_INODE_START <= inode_id && inode_id <= QUERY_INODE_END
}

pub const QUERY_INODE_START: u64 = FUSE_ROOT_ID + 1;
pub const ANY_QUERY_INODE: u64 = QUERY_INODE_START;

pub const QUERY_INODE_END: u64 = 100;
