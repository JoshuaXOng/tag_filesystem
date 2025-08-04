use std::{collections::HashMap, time::SystemTime};

use fuser::{FileAttr, FileType};

use crate::{entries::TfsEntry, inodes::{QueryInode, TagInodes, ANY_QUERY_INODE}};

#[derive(Debug)]
pub struct TfsQueries {
    queries: HashMap<QueryInode, TagInodes>,
    next: QueryInode
}

impl TfsQueries {
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
            next: QueryInode::try_from(ANY_QUERY_INODE)
                .expect("`{ANY_QUERY_INODE}` to be a \
                    valid query inode.")
        }
    }

    // TODO: Think about the naming
    pub fn as_map(&self)
    -> &HashMap<QueryInode, TagInodes> { 
        &self.queries 
    }

    // TODO: Align naming of query in rest of code.
    pub fn insert_limited(&mut self, tfs_query: TagInodes)
    -> QueryInode {
        let current = self.next;
        self.queries.insert(current, tfs_query);
        self.next = current.get_next();
        current
    }
}

pub fn get_fuse_attributes(query_inode: &QueryInode)
-> FileAttr {
    FileAttr {
        ino: query_inode.get_id(),
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
        flags: 0,
        blksize: 512,
    }
}
