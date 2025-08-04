use std::ffi::{OsStr, OsString};

use fuser::{Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request, FUSE_ROOT_ID};
use tracing::{debug, instrument};

use crate::{filesystem::{get_dir_attributes, GraphFilesystem}, ttl::{ANY_TTL, NO_TTL}};

impl Filesystem for GraphFilesystem {
    #[instrument(skip(self, _request, _mask, reply))]
    fn access(
        &mut self, _request: &Request<'_>, 
        inode_id: u64, _mask: i32, reply: ReplyEmpty
    ) {
        let gfs_entry = match self.inode_id_to_gfs_entry.get(&inode_id) {
            Some(x) => x,
            None => {
                reply.error(libc::ENOENT); // TODO: Use better error.
                return;
            }
        };

        // TODO: Do some checking to see if autho'd I guess.
        reply.ok();
    }

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
        if parent_inode != FUSE_ROOT_ID {
            reply.error(libc::ENOENT); // TODO: Use better error.
            return;
        }

        let file_attributes = self.create_file(file_name);
        // TODO: What does TTL, generation, fh, flags do here?
        let generation = 0;
        let fh = 0;
        let flags = 0;
        reply.created(&ANY_TTL, &file_attributes, generation, fh, flags);
    }

    #[instrument(skip_all, fields(parent_inode, name))]
    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent_inode: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        if parent_inode != FUSE_ROOT_ID {
            reply.error(libc::ENOENT); // TODO: Use better error.
            return;
        }
        self.create_tag(name);
    }

    /// Cases to consider, e.g.:
    /// - `/tmp/<GFS_MOUNT>$ ls` 
    /// - `/tmp/<GFS_MOUNT>$ ls "{ tag_1, tag_2 }"` 
    /// - `/tmp/<GFS_MOUNT>/{ tag_1 }$ ls "{ tag_2 }"` 
    #[instrument(skip(self, reply))]
    fn lookup(
        &mut self, _: &Request, parent_inode: u64,
        name_predicate: &std::ffi::OsStr, reply: ReplyEntry
    ) {
        if parent_inode != FUSE_ROOT_ID {
            reply.error(libc::ENOENT);
            return;
        }

        let is_tag_set = if name_predicate.to_string_lossy()
            .chars().next() == Some('{') { true } else { false };
        debug!("Is a tag? = `{is_tag_set}`.");
        if is_tag_set {
            let entry_query = OsString::from(name_predicate);
            self.query_for_entries = Some((entry_query, Self::ENTRIES_QUERY_ATTRIBUTES));
            reply.entry(&NO_TTL, &get_dir_attributes(Self::ENTRIES_QUERY_ATTRIBUTES.ino), 0);
            return;
        }

        let entry_payload = self.get_entry_by_name(name_predicate) ;
        debug!("Entry found = {:?}", entry_payload);
        match entry_payload {
            Some(x) => reply.entry(&x.get_ttl(), x.get_attributes(), 0),
            None => reply.error(libc::ENOENT) // TODO: Logging
        };
    }

    #[instrument(skip_all, fields(inode_id, _fh))]
    fn getattr(
        &mut self,
        _req: &Request<'_>,
        inode_id: u64,
        _fh: Option<u64>,
        reply: ReplyAttr,
    ) {
        debug!("Running `getattr`, inode = `{inode_id}`.");
        if inode_id == FUSE_ROOT_ID {
            reply.attr(&NO_TTL, &Self::ROOT_ATTRIBUTES);
            return;
        }
        
        if inode_id == Self::ENTRIES_QUERY_ATTRIBUTES.ino {
            reply.attr(&NO_TTL, &Self::ENTRIES_QUERY_ATTRIBUTES);
            return;
        }

        let entry_payload = self.inode_id_to_gfs_entry.get(&inode_id);
        debug!("Running `getattr`, fs entry = `{entry_payload:?}`.");
        let filesystem_entry = match entry_payload {
            Some(x) => x,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };
        reply.attr(&ANY_TTL, &filesystem_entry.get_attributes());
    }

    // TODO: Can't do skip_all.
    // Add to `tracing` crate potentially.
    #[instrument(skip(self, _req, reply))]
    fn readdir(
        &mut self,
        _req: &Request,
        inode_id: u64,
        _file_handle: u64,
        mut pagination_offset: i64,
        mut reply: ReplyDirectory,
    ) {
        debug!("Inode = `{inode_id}`, offset = `{pagination_offset}`.");
        if inode_id != Self::ENTRIES_QUERY_ATTRIBUTES.ino
        && inode_id != FUSE_ROOT_ID {
            reply.error(libc::ENOENT); // TODO: Right error?
            return;
        }

        if pagination_offset != 0 {
            reply.error(libc::ENOENT); // TODO: Handle
            return;
        }

        if inode_id == FUSE_ROOT_ID {
            for entry_payload in self.get_tag_entries().iter() {
                pagination_offset += 1;
                reply.add(
                    entry_payload.get_attributes().ino, pagination_offset,
                    entry_payload.get_attributes().kind,
                    entry_payload.get_name()
                );
            }
            reply.ok();
            return;
        }

        debug!("Entries query = `{:?}`.", self.query_for_entries);
        let entries_query = match &self.query_for_entries {
            Some(x) => x.0.clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            },
        };
        self.query_for_entries = None;
        for (entry_name, inode_id) in &self.entry_name_to_inode_id {
            pagination_offset += 1;
            debug!(
                "Offset = `{}`, name = `{}`, inode = `{}`.",
                pagination_offset, entry_name.to_string_lossy(), inode_id
            );

            let entry_payload = match self.get_entry_by_name(&entry_name) {
                Some(x) => x,
                None => continue, // TODO: Logging
            };

            if !self.does_query_match(&entries_query, &entry_payload) {
                continue;
            }

            reply.add(
                *inode_id, pagination_offset,
                entry_payload.get_attributes().kind, entry_name
            );
        }
        reply.ok();
    }

    #[instrument(skip(self, _request, reply))]
    fn write(
        &mut self,
        _request: &Request<'_>,
        inode_id: u64,
        fh: u64, // TODO: Figure out what is with this file handle opt.
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        let gfs_entry = match self.inode_id_to_gfs_entry.get_mut(&inode_id) {
            Some(x) => x,
            None => {
                reply.error(libc::ENOENT);
                return;
            },
        };
    }
}
