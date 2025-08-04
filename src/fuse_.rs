use std::{ffi::OsStr, time::SystemTime};

use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, ReplyEmpty, ReplyEntry, Request, TimeOrNow, FUSE_ROOT_ID};
use log::info;
use tracing::{error, instrument};

use crate::{entries::TfsEntry, filesystem::TagFilesystem, inodes::{is_inode_a_query, is_inode_a_tag, FileInode, QueryInode, TagInode, TagInodes}, queries::get_fuse_attributes, ttl::{ANY_TTL, NO_TTL}, unwrap_or};

// TODO: Check they reply errors are the most suitable ones.
// TODO: Errors need to be displayed to the user not just logged.
// TODO: What does TTL, generation, fh, flags do?
// TODO: Can't do skip_all.
//
// TODO: Implement Display
impl Filesystem for TagFilesystem {
    #[instrument(skip(self, _request, _mode, _umask, _flags, reply))]
    fn create(
        &mut self,
        _request: &Request<'_>,
        parent_inode: u64,
        file_name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        if parent_inode != FUSE_ROOT_ID && !is_inode_a_query(parent_inode) {
            error!("Not child of TFS root nor a query.");
            reply.error(libc::ENOENT);
            return;
        }

        let generation = 0;
        let fh = 0;
        let flags = 0;

        if parent_inode == FUSE_ROOT_ID {
            let new_file = unwrap_or!(self.create_file(file_name), e, {
                error!("{e}");
                reply.error(libc::ENOENT);
                return;
            });
            reply.created(
                &ANY_TTL, &new_file.get_attributes(),
                generation, fh, flags);
            info!("Created.");
            return;
        }

        let query_inode = unwrap_or!(QueryInode::try_from(parent_inode), e, {
            error!("{e}");
            reply.error(libc::ENOENT);
            return;
        });
        let file_tags = unwrap_or!(self.get_queries().as_map().get(&query_inode), {
            error!("Query with id `{query_inode:?}` does not exist.");
            reply.error(libc::ENOENT);
            return;
        });

        let new_file = unwrap_or!(
            self.create_file_with_tags(file_name, file_tags.clone()),
            e, {
                error!("{e}");
                reply.error(libc::ENOENT);
                return;
            }
        );
        reply.created(&ANY_TTL, &new_file.get_attributes(), generation, fh, flags);
        info!("Created.");
    }

    #[instrument(skip(self, _request, _mode, _umask, reply))]
    fn mkdir(
        &mut self,
        _request: &Request<'_>,
        parent_inode: u64,
        tag_name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        if parent_inode != FUSE_ROOT_ID {
            error!("Not child of TFS root.");
            reply.error(libc::ENOENT);
            return;
        }

        let new_tag = unwrap_or!(self.create_tag(tag_name), e, {
            error!("{e}");
            reply.error(libc::ENOENT);
            return;
        });
        let generation = 0;
        reply.entry(&ANY_TTL, &new_tag.get_attributes(), generation);
        info!("Created.");
    }
    
    // Predicate is overloaded, expected to be a name (for file or tag) or
    // a tag set (query for files), e.g., `{ tag_1, tag_2, tag_3 }`.
    #[instrument(skip(self, reply))]
    fn lookup(
        &mut self, _: &Request, parent_inode: u64,
        predicate: &std::ffi::OsStr, reply: ReplyEntry
    ) {
        let should_store_query = predicate.to_string_lossy()
            .chars()
            .next() == Some('{');
        if should_store_query {
            let file_query = unwrap_or!(predicate.try_into(), e, {
                error!("Failed to store query. {e}");
                reply.error(libc::ENOENT);
                return;
            });
            let query_inode = unwrap_or!(self.insert_query(file_query), e, {
                error!("Query storing failed. {e}");
                reply.error(libc::ENOENT);
                return;
            });
            reply.entry(&NO_TTL, &get_fuse_attributes(&query_inode), 0);
            info!("Stored query.");
            return;
        }

        if let Ok(query_inode) = QueryInode::try_from(parent_inode) {
            let target_file = unwrap_or!(
                // TODO: See if can make more simple.
                self.get_file_by_name_and_query_inode(predicate, &query_inode)
                    .and_then(|file| {
                        file.ok_or(format!("File `{predicate:?}` not exist, \
                            query inode `{query_inode:?}`.").into()) }),
                e, {
                    error!("Failed to query file by tags. {e}");
                    reply.error(libc::ENOENT);
                    return;
                }
            );
            reply.entry(&NO_TTL, &target_file.get_attributes(), 0);
            info!("Executed query.");
            return;
        } 

        if parent_inode != FUSE_ROOT_ID {
            error!("Encountered bug, expecting inode `{FUSE_ROOT_ID}`.");
            reply.error(libc::ENOENT);
            return;
        }

        // TODO: Handle when file and tag are the same.
        let target_tag = unwrap_or!(self.get_tags().get_by_name(predicate), {
            error!("No matches for tag.");
            return (_ = reply.error(libc::ENOENT))
        });
        reply.entry(&ANY_TTL, &target_tag.get_attributes(), 0);
        info!("Executed tag lookup.")
        // TODO: see if setting query to None after this is appropriate
    }

    #[instrument(skip(self, _request, _file_handle, reply))]
    fn getattr(
        &mut self,
        _request: &Request<'_>,
        inode_id: u64,
        _file_handle: Option<u64>,
        reply: ReplyAttr,
    ) {
        // TODO: See impact of NO_TTL
        if inode_id == FUSE_ROOT_ID {
            reply.attr(&NO_TTL, &ROOT_ATTRIBUTES);
            info!("Replied.");
            return;
        }
        
        if let Ok(query_inode) = QueryInode::try_from(inode_id) {
            reply.attr(&NO_TTL, &get_fuse_attributes(&query_inode));
            info!("Replied.");
            return;
        }

        if let Some(attributes) = self.get_attributes(inode_id) {
            reply.attr(&ANY_TTL, &attributes);
            info!("Replied.");
            return;
        };

        error!("Did not match any inode.");
        reply.error(libc::ENOENT);
    }

    #[instrument(skip(self, _request, _file_handle, reply))]
    fn readdir(
        &mut self,
        _request: &Request,
        inode_id: u64,
        _file_handle: u64,
        mut pagination_offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if is_inode_a_tag(inode_id) {
            reply.ok();
            info!("Not listing as it is a tag.");
            return;
        }

        let is_listing_root = inode_id == FUSE_ROOT_ID;
        if !is_listing_root && !is_inode_a_query(inode_id) {
            error!("Not root or a query.");
            reply.error(libc::ENOENT);
            return;
        }

        let pagination_offset_: usize = unwrap_or!(
            pagination_offset.try_into(),
            e, {
                error!("Can't convert offset. {e}");
                reply.error(libc::EINVAL);
                return;
            }
        );

        let add = |
            tfs_entry: &dyn TfsEntry, pagination_offset: i64,
            reply: &mut ReplyDirectory
        | {
            let had_space = reply.add(
                tfs_entry.get_raw_inode(), pagination_offset,
                tfs_entry.get_attributes().kind,
                tfs_entry.get_name()
            );
            had_space
        };

        if is_listing_root {
            let mut tagless_files = self.get_files().get_by_tags(&TagInodes::new());
            tagless_files.sort();
            let tagless_files = tagless_files.into_iter().map(|file| file as &dyn TfsEntry);

            let mut all_tags = self.get_tags().get_all();
            all_tags.sort();
            let all_tags = all_tags.into_iter().map(|tag| tag as &dyn TfsEntry);

            for tfs_entry in tagless_files.chain(all_tags).skip(pagination_offset_) {
                pagination_offset += 1;
                let had_space = add(tfs_entry, pagination_offset, &mut reply);
                if !had_space {
                    reply.ok();
                    info!("Partially listed root.");
                    return;
                }
            }

            reply.ok();
            info!("Finished listing root.");
            return;
        }

        let mut queried_files = unwrap_or!(
            QueryInode::try_from(inode_id)
                .and_then(|inode| self.get_files_by_query_inode(&inode)),
            e, {
                error!("{e}");
                reply.error(libc::EINVAL);
                return;
            });

        queried_files.sort();

        for queried_file in queried_files.into_iter().skip(pagination_offset_) {
            pagination_offset += 1;
            let had_space = add(queried_file, pagination_offset, &mut reply);
            if !had_space {
                reply.ok();
                info!("Partially finished query.");
                return;
            }
        }

        reply.ok();
        info!("Finished query.");
    }

    #[instrument(skip(
        self, _request, _mode, uid, gid, size, _atime,
        _mtime, _ctime, fh, _crtime, _chgtime, _bkuptime, flags, reply
    ))]
    fn setattr(
        &mut self,
        _request: &Request<'_>,
        target_inode: u64,
        _mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        // TODO
        if let Some(attributes) = self.get_attributes(target_inode) {
            reply.attr(&ANY_TTL, &attributes);
            info!("Set attributes.");
        } else {
            error!("Inode does not match anything.");
            reply.error(libc::ENOENT);
        }
    }

    #[instrument(skip(self, _request, reply))]
    fn unlink(
        &mut self,
        _request: &Request<'_>,
        parent_inode: u64,
        file_name: &OsStr,
        reply: ReplyEmpty,
    ) {
        if parent_inode != FUSE_ROOT_ID && !is_inode_a_query(parent_inode) {
            error!("Not child of TFS root nor a query.");
            reply.error(libc::ENOENT);
            return;
        }

        if parent_inode == FUSE_ROOT_ID {
            self.remove_file_by_name_and_tags(file_name, &TagInodes::new());
        }

        if let Ok(query_inode) = QueryInode::try_from(parent_inode) {
            if let Err(e) = self.remove_file_by_name_and_query_inode(
                file_name, &query_inode)
            {
                error!("{e}"); 
                reply.error(libc::ENOENT);
                return;
            };
        }

        reply.ok();
        info!("Deleted.");
    }


    #[instrument(skip(self, _request, reply))]
    fn rmdir(
        &mut self,
        _request: &Request<'_>,
        parent_inode: u64,
        tag_name: &OsStr,
        reply: ReplyEmpty,
    ) {
        if parent_inode != FUSE_ROOT_ID {
            error!("Not child of TFS root.");
            reply.error(libc::ENOENT);
            return;
        }
        
        if let Err(e) = self.delete_tag(tag_name) {
            error!("Failed to delete tag. {e}");
            reply.error(libc::ENOENT);
            return;
        }
        reply.ok();
        info!("Deleted.");
    }
}

impl TagFilesystem {
    pub fn get_attributes(&self, inode_id: u64) -> Option<FileAttr> {
        if inode_id == FUSE_ROOT_ID {
            return Some(ROOT_ATTRIBUTES);
        }

        if let Ok(query_inode) = QueryInode::try_from(inode_id) {
            return Some(get_fuse_attributes(&query_inode));
        }

        if let Some(attributes) = FileInode::try_from(inode_id)
            .ok()
            .and_then(|inode| self.get_files().get_by_inode(&inode))
            .map(|file| file.get_attributes())
        {
            return Some(attributes);
        };

        if let Some(attributes) = TagInode::try_from(inode_id)
            .ok()
            .and_then(|inode| self.get_tags().get_by_inode(&inode))
            .map(|tag| tag.get_attributes())
        {
            return Some(attributes);
        };

        None
    } 

    fn get_entries_sorted(&self) -> impl Iterator<Item = &dyn TfsEntry> {
        let mut all_tags = self.get_tags().get_all();
        all_tags.sort();
        let all_tags = all_tags.into_iter().map(|tag| tag as &dyn TfsEntry);

        let mut all_files = self.get_files().get_all();
        all_files.sort();
        let all_files = all_files.into_iter().map(|file| file as &dyn TfsEntry);
        
        all_tags.chain(all_files)
    }
}

const ROOT_ATTRIBUTES: FileAttr = FileAttr {
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
