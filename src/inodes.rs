use std::{collections::BTreeSet, fmt::Display};

use fuser::FUSE_ROOT_ID;
use rand::random_range;

use crate::{errors::{AnyError, ResultBtAny}, unwrap_or,
    wrappers::write_btreeset, WithBacktrace};

// TODO: Add numbers to tabs in VIM.

const CUSTOM_INODE_START: u64 = FUSE_ROOT_ID + 1;

const INODE_TYPE_COUNT: u64 = 3;

const FILE_TYPE_REMAINDER: u64 = 0;
const TAG_TYPE_REMAINDER: u64 = 1;
const NAMESPACE_TYPE_REMAINDER: u64 = 2;

fn get_is_inode_type(inode_id: u64, type_remainder: u64) -> bool {
    CUSTOM_INODE_START <= inode_id && inode_id % INODE_TYPE_COUNT == type_remainder
}

pub const ANY_NAMESPACE_INODE_: u64 = FUSE_ROOT_ID + 1;

pub fn get_is_inode_root(inode_id: u64) -> bool {
    inode_id == FUSE_ROOT_ID
}

fn generate_jumpoff_inode() -> u64 {
    let unscaled_start = CUSTOM_INODE_START / INODE_TYPE_COUNT;
    let unscaled_end = (u64::MAX) / INODE_TYPE_COUNT;
    random_range(unscaled_start..=unscaled_end) * INODE_TYPE_COUNT
}

fn generate_free_inode<T>(inodes_inuse: Vec<&T>, type_remainder: u64)
    -> ResultBtAny<T> where T: TryFrom<u64> + PartialEq + Ord
{
    loop {
        let inode_id = generate_jumpoff_inode() + type_remainder;
        let candidate_inode = unwrap_or!(T::try_from(inode_id), _e, continue);
        if !inodes_inuse.contains(&&candidate_inode) {
            return Ok(candidate_inode);
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct FileInode {
    id: u64,
}

impl FileInode {
    pub fn get_id(&self) -> u64 { self.id }

    pub fn get_is_file(inode_id: u64) -> bool {
        get_is_inode_type(inode_id, FILE_TYPE_REMAINDER)
    }

    pub fn try_from_free_inodes<T>(inodes_inuse: Vec<&T>)
    -> ResultBtAny<T> where T: TryFrom<u64> + PartialEq + Ord {
        generate_free_inode(inodes_inuse, FILE_TYPE_REMAINDER)
    }
}

impl TryFrom<u64> for FileInode {
    type Error = WithBacktrace<AnyError>;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !Self::get_is_file(value) {
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

    pub fn get_is_tag(inode_id: u64) -> bool {
        get_is_inode_type(inode_id, TAG_TYPE_REMAINDER)
    }

    pub fn try_from_free_inodes<T>(inodes_inuse: Vec<&T>)
    -> ResultBtAny<T> where T: TryFrom<u64> + PartialEq + Ord {
        generate_free_inode(inodes_inuse, TAG_TYPE_REMAINDER)
    }
}

impl TryFrom<u64> for TagInode {
    type Error = WithBacktrace<AnyError>;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !Self::get_is_tag(value) {
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

impl<T> From<T> for TagInodes where T: Iterator<Item = TagInode> {
    fn from(value: T) -> Self {
        let mut tag_inodes = Self::new();
        value.for_each(|inode| _ = tag_inodes.0.insert(inode));
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

    pub fn get_is_namespace(inode_id: u64) -> bool {
        get_is_inode_type(inode_id, NAMESPACE_TYPE_REMAINDER)
    }

    pub fn try_from_free_inodes<T>(inodes_inuse: Vec<&T>)
    -> ResultBtAny<T> where T: TryFrom<u64> + PartialEq + Ord {
        generate_free_inode(inodes_inuse, NAMESPACE_TYPE_REMAINDER)
    }
}

impl Display for NamespaceInode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl TryFrom<u64> for NamespaceInode {
    type Error = WithBacktrace<AnyError>;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !Self::get_is_namespace(value) {
            return Err(format!("Not a valid namespace inode value `{value}`.").into()); 
        }
        Ok(Self { id: value })
    }
}

#[test]
fn checking_any_namespace_is_valid() {
    assert!(NamespaceInode::get_is_namespace(ANY_NAMESPACE_INODE_));
}
