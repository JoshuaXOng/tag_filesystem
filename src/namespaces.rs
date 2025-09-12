use std::{collections::HashMap, fmt::Display, time::SystemTime};

use fuser::{FileAttr, FileType};

use crate::{errors::Result_, inodes::{NamespaceInode, TagInodes, ANY_NAMESPACE_INODE}, wrappers::write_iter};

#[derive(Debug)]
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
    namespaces: HashMap<NamespaceInode, TfsNamespace>,
    next: NamespaceInode
}

impl IndexedNamepsaces {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            next: NamespaceInode::try_from(ANY_NAMESPACE_INODE)
                .expect("`ANY_NAMESPACE_INODE` to be a valid namespace inode.")
        }
    }

    pub fn get_by_inode(&self, namespace_inode: &NamespaceInode) -> Result_<&TfsNamespace> {
        self.namespaces.get(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode).into())
    }

    pub fn get_by_inode_id(&self, inode_id: u64) -> Result_<&TfsNamespace> {
        let namespace_inode = NamespaceInode::try_from(inode_id)?;
        self.namespaces.get(&namespace_inode)
            .ok_or(format!("Namespace with inode `{namespace_inode}` does not \
                exist.").into())
    }

    pub fn get_by_inode_mut(&mut self, namespace_inode: &NamespaceInode)
    -> Result_<NamespaceUpdate<'_>> {
        Ok(self.namespaces
            .get_mut(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode))?
            .into())
    }

    fn get_namespace_404_message(namespace_inode: &NamespaceInode) -> String {
        format!("Namespace id `{namespace_inode}` does not exist.")
    }

    pub fn get_all(&self) -> &HashMap<NamespaceInode, TfsNamespace> { 
        &self.namespaces 
    }

    // TODO: Might have to turn it into a series as a pre-req for safely
    // allowing multi depth namespaces.
    pub fn insert_limited(&mut self, namespace_name: impl Into<String>, namespace_tags: TagInodes)
    -> NamespaceInode {
        let current = self.next;
        self.namespaces.insert(current, TfsNamespace {
            name: namespace_name.into(),
            inode: current,
            tags: namespace_tags
        });
        self.next = current.get_next();
        current
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
