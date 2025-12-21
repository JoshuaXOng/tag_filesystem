use std::{collections::HashMap, fmt::Display, time::SystemTime};

use bon::Builder;
use fuser::{FileAttr, FileType};

use crate::{errors::ResultBtAny, inodes::{NamespaceInode, TagInodes},
    wrappers::write_iter};

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct TfsNamespace {
    pub name: String,
    pub inode: NamespaceInode,
    pub tags: TagInodes
}

impl<'a> From<&'a TfsNamespace> for &'a TagInodes {
    fn from(value: &'a TfsNamespace) -> Self {
        &value.tags
    }
}

impl From<TfsNamespace> for TagInodes {
    fn from(value: TfsNamespace) -> Self {
        value.tags
    }
}

impl Display for TfsNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(id={}, tags={})", self.name, self.inode, self.tags)
    }
}

#[derive(Debug)]
pub struct IndexedNamepsaces {
    namespaces: HashMap<NamespaceInode, TfsNamespace>
}

impl IndexedNamepsaces {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new()
        }
    }

    pub fn get_by_inode(&self, namespace_inode: &NamespaceInode) -> ResultBtAny<&TfsNamespace> {
        self.namespaces.get(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode).into())
    }

    pub fn get_by_inode_id(&self, inode_id: u64) -> ResultBtAny<&TfsNamespace> {
        let namespace_inode = NamespaceInode::try_from(inode_id)?;
        self.namespaces.get(&namespace_inode)
            .ok_or(format!("Namespace with inode `{namespace_inode}` does not \
                exist.").into())
    }

    pub fn get_by_inode_mut(&mut self, namespace_inode: &NamespaceInode)
    -> ResultBtAny<NamespaceUpdate<'_>> {
        Ok(self.namespaces
            .get_mut(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode))?
            .into())
    }

    fn get_namespace_404_message(namespace_inode: &NamespaceInode) -> String {
        format!("Namespace id `{namespace_inode}` does not exist.")
    }

    pub fn get_all(&self) -> Vec<&TfsNamespace> {
        self.namespaces.values().collect()
    }

    pub fn get_map(&self) -> &HashMap<NamespaceInode, TfsNamespace> { 
        &self.namespaces 
    }

    pub fn get_free_inode(&self) -> ResultBtAny<NamespaceInode> {
        let inodes_inuse = self.namespaces.keys().collect();
        NamespaceInode::try_from_free_inodes(inodes_inuse)
    }

    pub fn add(&mut self, to_add: TfsNamespace) -> ResultBtAny<NamespaceInode> {
        let namespace_inode = to_add.inode;

        let does_conflict = self.namespaces.get(&namespace_inode).is_some();
        if does_conflict {
            Err(format!("Namespace with id `{}` already exists.", namespace_inode))?;
        }
        
        self.namespaces.insert(namespace_inode, to_add);
        Ok(namespace_inode)
    }

    pub fn do_for_all<T>(&mut self, mut to_do: impl FnMut(NamespaceUpdate) -> T) -> Vec<T> {
        let mut to_return = Vec::new();
        for tfs_namespace in self.namespaces.values_mut() {
            to_return.push(to_do(tfs_namespace.into()));
        }
        to_return
    }
}

impl Display for IndexedNamepsaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_iter(f, ('[', ']'), self.namespaces.values())
    }
}

pub struct NamespaceUpdate<'a> {
    pub name: &'a mut String,
    inode: &'a NamespaceInode,
    pub tags: &'a mut TagInodes
}

impl<'a> NamespaceUpdate<'a> {
    pub fn get_inode(&self) -> &NamespaceInode {
        &self.inode
    }
}

impl<'a> From<&'a mut TfsNamespace> for NamespaceUpdate<'a> {
    fn from(value: &'a mut TfsNamespace) -> Self {
        NamespaceUpdate {
            name: &mut value.name,
            inode: &mut value.inode,
            tags: &mut value.tags
        }
    }
}

// TODO: Give proper values
pub fn get_fuse_attributes(namespace_inode: &NamespaceInode) -> FileAttr {
    FileAttr {
        ino: namespace_inode.get_id(),
        size: 0,
        blocks: 0,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::Directory,
        perm: 0o644,
        nlink: 1,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        blksize: 0,
        flags: 0,
    }
}
