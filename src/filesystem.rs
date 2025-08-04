use std::{collections::{HashMap, HashSet}, ffi::{OsStr, OsString}, fmt, time::{Duration, SystemTime}};

use fuser::{FileAttr, FileType, FUSE_ROOT_ID};
use tracing::instrument;

use crate::{errors::Result_, ttl::ANY_TTL};

type FileInode = u64;
type TagInode = u64;

#[derive(Debug)]
enum GfsInode {
    FileInode(FileInode),
    TagInode(TagInode)
}

#[derive(Debug)]
struct GfsFile {
}

#[derive(Debug)]
struct GfsTag {
}

// TODO: Don't use enum lol.
#[derive(Debug)]
pub enum GfsEntry {
    File {
        name: OsString,
        attributes: FileAttr,
        content: Vec<u8>,
        tags: HashSet<u64>
    },
    Tag {
        name: OsString,
        attributes: FileAttr
    }
}

impl GfsEntry {
    pub fn get_attributes(&self) -> &FileAttr {
        match self {
            GfsEntry::File { attributes, .. } => attributes,
            GfsEntry::Tag { attributes, .. } => attributes,
        }
    }

    pub fn get_name(&self) -> &OsString {
        match self {
            GfsEntry::File { name, .. } => name,
            GfsEntry::Tag { name, .. } => name,
        }
    }

    pub fn get_inode_id(&self) -> u64 {
        match self {
            GfsEntry::File { attributes, .. } => attributes.ino,
            GfsEntry::Tag { attributes, .. } => attributes.ino
        }
    }

    pub fn get_ttl(&self) -> Duration {
        ANY_TTL
    }
}

/// Root inode id is 1.
/// Query inode is is 2. 
/// File inodes are >2 and odd.
/// Tag inodes are >3 and even.
/// Entries are files or tags.
///
/// `ls` shows files that match current working tags,
/// and other tags that'll  lead to more file.
///
/// TODO: Confirm if not compatible with cd
/// - `/tmp/<GFS_MOUNT>/{ tag_1 }$ cd "{ tag_2 }"` 
///   should result in `/tmp/<GFS_MOUNT>/{ tag_1, tag_2 }$
/// Might need to provide a `cd` equivalent bin/script.
#[derive(Debug)]
pub struct GraphFilesystem {
    files: HashMap<FileInode, GfsFile>,
    tags: HashMap<TagInode, GfsTag>,
    name_to_inode: HashMap<OsString, GfsInode>,

    pub inode_id_to_gfs_entry: HashMap<u64, GfsEntry>,
    // TODO: Key needs to be name + tags -> inode
    pub entry_name_to_inode_id: HashMap<OsString, u64>,
    pub query_for_entries: Option<(OsString, FileAttr)>,
}

impl GraphFilesystem {
    pub const ENTRIES_QUERY_ATTRIBUTES: FileAttr = FileAttr {
        ino: 2,
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
    };

    pub const ROOT_ATTRIBUTES: FileAttr = FileAttr {
        ino: FUSE_ROOT_ID,
        size: 0,
        blocks: 0,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::Directory,
        perm: 0o755,
        nlink: 2,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        flags: 0,
        blksize: 512,
    };

    pub fn new() -> Self {
        GraphFilesystem {
            inode_id_to_gfs_entry: HashMap::new(),
            entry_name_to_inode_id: HashMap::new(),
            query_for_entries: None,
        }
    }

    #[instrument(skip(self))]
    pub fn get_entry_by_name(&self, entry_name: &OsStr) -> Option<&GfsEntry> {
        let entry_inode = self.entry_name_to_inode_id.get(entry_name)?;
        self.inode_id_to_gfs_entry.get(entry_inode)
    }

    #[instrument(skip(self))]
    fn get_free_file_inode(&self) -> u64 { // TODO: Binary search for empty space.
        let inodes_inuse = self.get_inodes_inuse();
        for inode_id in 3.. {
            if !Self::is_inode_a_file(inode_id) {
                continue;
            }

            let is_used = inodes_inuse.contains(&inode_id);
            if !is_used {
                return inode_id;
            }
        }
        0 // TODO: Sort this out.
    }

    #[instrument(skip(self))]
    fn get_free_tag_inode(&self) -> u64 {
        let inodes_inuse = self.get_inodes_inuse();
        for inode_id in 3.. {
            if !Self::is_inode_a_tag(inode_id) {
                continue;
            }

            let is_used = inodes_inuse.contains(&inode_id);
            if !is_used {
                return inode_id;
            }
        }
        0 // TODO: Sort this out.
    }

    #[instrument(skip(self))]
    fn get_inodes_inuse(&self) -> HashSet<u64> {
        let mut inodes_inuse = HashSet::new();
        for (_, entry_payload) in self.inode_id_to_gfs_entry.iter() {
            let inode_id = entry_payload.get_attributes().ino;
            inodes_inuse.insert(inode_id);
        };
        inodes_inuse
    }

    #[instrument(skip(self))]
    pub fn get_tag_entries(&self) -> Vec<&GfsEntry> {
        let mut tag_entries = vec![];
        for (inode_id, entry_payload) in &self.inode_id_to_gfs_entry {
            if Self::is_inode_a_tag(*inode_id) {
                tag_entries.push(entry_payload); 
            }
        }
        tag_entries
    }

    #[instrument(skip(self))]
    pub fn create_file(&mut self, file_name: impl Into<OsString> + fmt::Debug) -> FileAttr {
        let available_inode = self.get_free_file_inode();
        let file_attributes = get_file_attributes(available_inode);
        self.create_gfs_entry(GfsEntry::File {
            name: file_name.into(),
            attributes: file_attributes,
            content: Vec::new(),
            tags: HashSet::new(),
        });
        file_attributes
    }

    #[instrument(skip(self))]
    pub fn create_tag(&mut self, tag_name: impl Into<OsString> + fmt::Debug) -> FileAttr {
        let available_inode = self.get_free_tag_inode();
        let file_attributes = get_file_attributes(available_inode);
        self.create_gfs_entry(GfsEntry::Tag {
            name: tag_name.into(),
            attributes: file_attributes
        });
        file_attributes 
    }

    #[instrument(skip(self))]
    fn create_gfs_entry(&mut self, gfs_entry: GfsEntry) {
        let available_inode = gfs_entry.get_inode_id();
        self.entry_name_to_inode_id
            .insert(gfs_entry.get_name().to_owned(), available_inode);
        self.inode_id_to_gfs_entry
            .insert(available_inode, gfs_entry);
    }

    #[instrument(skip(self))]
    pub fn add_tag_to_file(&mut self, file_inode: u64, tag_inode: u64)
    -> Result_<()> {
        let error_span = "Failed to add tag to file";
        
        let file_entry = match self.inode_id_to_gfs_entry.get_mut(&file_inode) {
            Some(x) => x,
            None => Err(format!( // TODO: Check whitespaces
                "{error_span}, inode `{file_inode}` \
                does not event exist."
            ))?
        };
        match file_entry {
            GfsEntry::File { tags, .. } => tags.insert(tag_inode),
            GfsEntry::Tag { name, .. } => Err(format!(
                "{error_span}, inode `{file_inode}` does not 
                corresponds with a tag (with name `{name:?}`."
            ))?
        };

        Ok(())
    }

    fn is_inode_a_file(inode_id: u64) -> bool {
        inode_id > 2 && inode_id % 2 != 0
    }

    fn is_inode_a_tag(inode_id: u64) -> bool {
        inode_id > 2 && inode_id % 2 == 0
    }

    #[instrument(skip(self))]
    pub fn does_query_match(&self, entries_query: &OsStr, entry_payload: &GfsEntry) -> bool {
        let mut _entries_query = HashSet::new();
        let x = entries_query.to_string_lossy();
        let entries_query = x
            .trim_matches(|character| {
                match character {
                    '{' | '}' | ' ' => true,
                    _ => false
                }
            })
            .split(',');
        for todo in entries_query {
            match self.entry_name_to_inode_id.get(OsStr::new(todo)) { // TODO: Logging
                Some(x) => _entries_query.insert(*x),
                None => return false, // TODO: Handle better 
            };
        }

        match entry_payload {
            GfsEntry::File { tags, .. } => {
                if _entries_query == *tags { true } else { false}
            },
            GfsEntry::Tag { .. } => false, // TODO: Logging.
        }
    }
}

pub fn get_file_attributes(inode_id: u64) -> FileAttr {
    FileAttr {
        ino: inode_id,
        size: 12,
        blocks: 1,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind: FileType::RegularFile,
        perm: 0o644,
        nlink: 1,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        flags: 0,
        blksize: 512,
    }
}

pub fn get_dir_attributes(inode_id: u64) -> FileAttr {
    FileAttr {
        ino: inode_id,
        size: 12,
        blocks: 1,
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
