use std::{ffi::OsStr, thread::sleep, time::{Duration, SystemTime}};

use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyWrite, Request, TimeOrNow, FUSE_ROOT_ID};
use libc::{EINVAL, ENOENT};
use log::info;
use tracing::{error, instrument, warn};

use crate::{ResultExt2, entries::TfsEntry, errors::StringExt, files::TfsFile,
    filesystem::TagFilesystem, inodes::{get_is_inode_a_namespace, get_is_inode_root,
    FileInode, NamespaceInode, TagInode, TagInodes}, namespaces, storage::TfsStorage,
    tags::TfsTag, ttl::{ANY_TTL, NO_TTL}, unwrap_or};

// TODO: How to minimize error returning propagation, a lot of Options
// Maybe unwrap or should prefix with code in first param
macro_rules! returne {
    ($reply: ident, $error_code: expr, $log_level: ident, $error_message: expr, $($message_arguments: expr), *) => {{
        $log_level!($error_message, $($message_arguments), *);
        $reply.error($error_code);
        return;
    }};
    ($reply: ident, $error_code: expr, $log_level: ident, $error_message: expr) => {{
        returne!($reply, $error_code, $log_level, $error_message,)
    }};
}

// TODO: Some reply.error should not really log as an error.

// TODO: Sometimes the below error for `ct tag_2` when tag2 does exist.
// Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
// 
// TODO: Should have 2 (or infinite) depth query sets? Cause like { tag_1, tag_2 }/file_1 
// want to add tag_3, how to do with good ui
// cwd at { tag_1, tag_2 }, mv file_1 ./{ tag_3 }
// and want to remove a tag, mv file_1 ./{ ~tag_2 }

// TODO: Check they reply errors are the most suitable ones.
// TODO: Errors need to be displayed to the user not just logged.
// TODO: What does TTL, generation, fh, flags do?
// TODO: Make some of the FUSE ops atomic
impl<Storage: TfsStorage> Filesystem for TagFilesystem<Storage> {
    #[instrument(skip_all, fields(?parent_inode, ?file_name))]
    fn create(&mut self, request: &Request<'_>, parent_inode: u64, file_name: &OsStr,
        _mode: u32, _umask: u32, _flags: i32, reply: ReplyCreate)
    {
        if !get_is_inode_root(parent_inode) && !get_is_inode_a_namespace(parent_inode) {
            returne!(reply, ENOENT, error, "Not child of TFS root nor a namespace.");
        }

        let file_name = file_name.to_string_lossy();
        let generation = 0;
        let fh = 0;
        let flags = 0;

        let new_file = TfsFile::builder()
            .name(file_name.clone())
            .inode(unwrap_or!(self.get_free_file_inode(),
                e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt())))
            .owner(request.uid())
            .group(request.gid());

        if get_is_inode_root(parent_inode) {
            let new_file = unwrap_or!(self.add_file(new_file.build()),
                e, returne!(reply, EINVAL, error, "{}", e.to_string_wbt()));
            let file_inode = new_file.inode;
            let fuser_attributes = unwrap_or!(self.get_file_fuser(&file_inode),
                e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
            reply.created(&ANY_TTL, &fuser_attributes, generation, fh, flags);
            info!("Created.");
            return;
        }

        let namespace_inode = unwrap_or!(NamespaceInode::try_from(parent_inode),
            e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        let tfs_namespace = unwrap_or!(self.get_namespaces()
            .get_map()
            .get(&namespace_inode), 
            returne!(reply, ENOENT, error, "Namespace with id `{namespace_inode}` \
                does not exist."));

        let new_file = new_file.tags(tfs_namespace.tags.clone());
        let new_file = unwrap_or!(self.add_file(new_file.build()),
            e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        let file_inode = new_file.inode; // TODO: Why is this needed... ughh
        let fuser_attributes = unwrap_or!(self.get_file_fuser(&file_inode),
            // TODO: More appropriate error code.
            e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        reply.created(&ANY_TTL, &fuser_attributes, generation, fh, flags);
        info!("Created.");
    }

    #[instrument(skip_all, fields(?parent_inode, ?tag_name))]
    fn mkdir(&mut self, request: &Request<'_>, parent_inode: u64, tag_name: &OsStr,
        _mode: u32, _umask: u32, reply: ReplyEntry)
    {
        let tag_name = tag_name.to_string_lossy();

        // TODO: Maybe accept mkdir everywhere, just always create at global.
        if !get_is_inode_root(parent_inode) {
            returne!(reply, ENOENT, error, "Needs to be under the root directory.");
        }

        let is_file_conflicting = self.get_files()
            .get_by_name_and_tags(&tag_name, &TagInodes::new())
            .is_some();
        if is_file_conflicting {
            returne!(reply, EINVAL, error, "Untagged file already exists with \
                name `{tag_name}`.");
        }

        let new_tag = TfsTag::builder()
            .name(tag_name)
            .inode(unwrap_or!(self.get_free_tag_inode(),
                e, returne!(reply, EINVAL, error,
                    "No free tag inode. {}", e.to_string_wbt())))
            .owner(request.uid())
            .group(request.gid())
            .build();
        let new_tag = unwrap_or!(self.add_tag(new_tag),
            e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        let tag_inode = new_tag.inode;
        let fuser_attributes = unwrap_or!(self.get_tag_fuser(&tag_inode),
            e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        let generation = 0;
        reply.entry(&ANY_TTL, &fuser_attributes, generation);
        info!("Created.");
    }
    
    #[instrument(skip_all, fields(?parent_inode, ?predicate))]
    fn lookup(&mut self, _: &Request, parent_inode: u64, predicate: &OsStr,
        reply: ReplyEntry)
    {
        // TODO: Is there not just a method that returns String instead of Cow?
        let predicate = predicate.to_string_lossy().to_string();

        if get_is_inode_root(parent_inode) {
            if get_is_a_namespace(&predicate) {
                let namespace_inode = unwrap_or!(self.insert_namespace(predicate),
                    e, returne!(reply, ENOENT, error,
                        "Namespace lookup failed. {}", e.to_string_wbt()));
                reply.entry(&NO_TTL, &namespaces::get_fuse_attributes(&namespace_inode), 0);
                info!("Finished namespace lookup under root.");
                return;
            }

            // TODO: Can this be put in function to have single source of truth?
            let target_inode = unwrap_or!(self.get_tags()
                .get_by_name(&predicate)
                .map(|tag| tag.inode.get_id())
                .or(self.get_files()
                    .get_by_name_and_tags(&predicate, &TagInodes::new())
                    .map(|file| file.inode.get_id())),
                returne!(reply, ENOENT, info, "Tag/file lookup failed, \
                    `{predicate}` is not a tag nor an untagged file."));

            let fuser_attributes = unwrap_or!(self.get_fuser_attributes(target_inode),
                e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
            reply.entry(&ANY_TTL, &fuser_attributes, 0);
            info!("Finished tag/file lookup under root.");
            // TODO: See if setting query to None after this is appropriate.
            return;
        }

        // TODO: Need to add checks as to what file and tag names can be created
        // to make them not conflict, no dupes when ls'ing
        if let Ok(parent_namespace) = self.get_namespaces().get_by_inode_id(parent_inode) {
            // TODO: Can this be put in function with twin-ish
            let fuser = unwrap_or!(self.get_inrange_tags(parent_namespace)
                .and_then(|tags| tags.into_iter()
                    .find(|tag| tag.name == predicate)
                    .ok_or(format!("No matching neighbour tag w/ name `{}` \
                        for namespace `{}`.", predicate, parent_namespace.inode).into()))
                .and_then(|tag| self.get_tag_fuser(&tag.inode))
                .or(self.get_file_by_name_and_namespace_inode(&predicate, &parent_namespace.inode)
                    .and_then(|file| self.get_file_fuser(&file.inode))),
                e, returne!(reply, ENOENT, error, "Tag/file lookup failed, \
                    `{predicate}` is not a tag not a file under inode \
                    `{parent_inode}`. {}", e.to_string_wbt()));
            // TODO: no magic variables.
            let generation = 0;
            reply.entry(&NO_TTL, &fuser, generation);
            info!("Finished tag/file lookup under namespace.");
            return;
        } 

        returne!(reply, ENOENT, error, "No tags or files matching `{predicate}` \
            under root and/or namespace, `{parent_inode}`.");
    }

    #[instrument(skip_all, fields(?inode_id))]
    fn getattr(&mut self, _request: &Request<'_>, inode_id: u64,
        _file_handle: Option<u64>, reply: ReplyAttr)
    {
        // TODO: See impact of NO_TTL
        // TODO: Use get_is_inode_root
        if get_is_inode_root(inode_id) {
            reply.attr(&NO_TTL, &ROOT_ATTRIBUTES);
            info!("Replied w/ root.");
            return;
        }
        
        if let Ok(namespace_inode) = NamespaceInode::try_from(inode_id) {
            reply.attr(&NO_TTL, &namespaces::get_fuse_attributes(&namespace_inode));
            info!("Replied w/ namespace.");
            return;
        }

        if let Ok(fuser_attributes) = FileInode::try_from(inode_id)
            .and_then(|inode| self.get_file_fuser(&inode))
            .or(TagInode::try_from(inode_id)
                .and_then(|inode| self.get_tag_fuser(&inode))) {
                    reply.attr(&ANY_TTL, &fuser_attributes);
                    info!("Replied w/ file.");
                    return;
                }

        error!("Did not match any inode.");
        reply.error(ENOENT);
    }

    // TODO: Should probably have to -f when deleting tags w/ files under them.
    // in `/tmp/tfs/` doing `rmdir tag_1` vs `rmdir "{ tag_1 }"
    // TODO: Should allow listing of root or only allow {}?
    // TODO: Determine if pagination can probably be race cond. in multi user
    #[instrument(skip_all, fields(?inode_id))]
    fn readdir(&mut self, _request: &Request, inode_id: u64, _file_handle: u64,
        mut pagination_offset: i64, mut reply: ReplyDirectory)
    {
        let is_listing_root = get_is_inode_root(inode_id);
        if !is_listing_root && !get_is_inode_a_namespace(inode_id) {
            returne!(reply, ENOENT, error, "Inode not root or a namespace.");
        }

        let pagination_offset_: usize = unwrap_or!(
            pagination_offset.try_into().with_bt(),
            e, returne!(reply, EINVAL, error, "Can't convert offset. {}", e.to_string_wbt()));

        let add = |tfs_entry: &dyn TfsEntry, pagination_offset: i64, reply: &mut ReplyDirectory| {
            reply.add(tfs_entry.get_inode_id(), pagination_offset,
                tfs_entry.get_file_kind(), tfs_entry.get_name())
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

        let current_namespace = unwrap_or!(
            self.get_namespaces().get_by_inode_id(inode_id),
            e, returne!(reply, EINVAL, error, "Could not get namespace. {}", e.to_string_wbt()));

        let mut inrange_tags = unwrap_or!(self.get_inrange_tags(current_namespace),
            e, returne!(reply, EINVAL, error, "Could not get inrange tags. {}", e.to_string_wbt()));
        inrange_tags.sort();
        let inrange_tags = inrange_tags.into_iter()
            .map(|tag| tag as &dyn TfsEntry);

        let mut inscope_files = unwrap_or!(
            self.get_files_by_namespace_inode(&current_namespace.inode),
            e, returne!(reply, EINVAL, error,
                "Could not get files under namespace. {}", e.to_string_wbt()));
        inscope_files.sort();
        let inscope_files = inscope_files.into_iter()
            .map(|file| file as &dyn TfsEntry);

        for to_list in inscope_files.chain(inrange_tags)
            .skip(pagination_offset_)
        {
            pagination_offset += 1;
            let had_space = add(to_list, pagination_offset, &mut reply);
            if !had_space {
                reply.ok();
                info!("Partially finished listing namespace.");
                return;
            }
        }

        reply.ok();
        info!("Finished finished listing namespace.");
    }

    // TODO: Use rest of args, or at least understand them.
    #[instrument(skip_all, fields(?target_inode, ?start_position, ?read_amount))]
    fn read(&mut self, _request: &Request<'_>, target_inode: u64, _file_handle: u64,
        start_position: i64, read_amount: u32, flags: i32, _lock_owner: Option<u64>,
        reply: ReplyData)
    {
        let file_inode: FileInode = unwrap_or!(target_inode.try_into(),
            e, returne!(reply, EINVAL, error, "Not a file inode. {}", e.to_string_wbt()));
        let start_position: u64 = unwrap_or!(start_position.try_into().with_bt(),
            e, returne!(reply, EINVAL, error, "Offset value can't be converted. {}", e.to_string_wbt()));
        let read_amount: usize = unwrap_or!(read_amount.try_into().with_bt(),
            e, returne!(reply, EINVAL, error, "Amount to read can't be converted. {}", e.to_string_wbt()));

        let content_read = unwrap_or!(self.get_storage()
            .read(&file_inode, start_position, read_amount),
            e, returne!(reply, EINVAL, error, "Failed to read file. {}", e.to_string_wbt()));

        reply.data(&content_read);
        info!("Read completed.");    
    }

    // TODO
    #[instrument(skip_all, fields(?target_inode))]
    fn open(&mut self, _request: &Request<'_>, target_inode: u64, _flags: i32,
        reply: ReplyOpen)
    {
        reply.opened(0, _flags as u32);
        info!("Opened.");
    }

    // TODO
    #[instrument(skip_all, fields(?target_inode))]
    fn flush(&mut self, _request: &Request<'_>, target_inode: u64, file_handle: u64,
        lock_owner: u64, reply: ReplyEmpty)
    {
        reply.ok();
        info!("Flushed.");
    }

    // TODO
    #[instrument(skip_all, fields(?target_inode))]
    fn fsyncdir(&mut self, _request: &Request<'_>, target_inode: u64,
        _file_handle: u64, _datasync: bool, reply: ReplyEmpty)
    {
        let should_fsync_all = get_is_inode_root(target_inode); 
        if should_fsync_all {
            unwrap_or!(self.save_persistently(),
                e, returne!(reply, EINVAL, error, "Failed to save TFS state. {}",
                    e.to_string_wbt()));
            reply.ok();
            info!("Saved all.");
            return;
        }
    }

    // TODO
    #[instrument(skip_all, fields(?_ino))]
    fn release(&mut self, _request: &Request<'_>, _ino: u64, _fh: u64, _flags: i32,
        _lock_owner: Option<u64>, _flush: bool, reply: ReplyEmpty)
    {
        reply.ok();
        info!("Released.");
    }

    #[instrument(skip_all, fields(?previous_parent, ?previous_name, ?new_parent, ?new_name))]
    fn rename(&mut self, _request: &Request<'_>, previous_parent: u64,
        previous_name: &OsStr, new_parent: u64, new_name: &OsStr, _flags: u32,
        reply: ReplyEmpty)
    {
        let previous_name = previous_name.to_string_lossy();
        let new_name = new_name.to_string_lossy().to_string();

        if get_is_inode_root(previous_parent) && get_is_inode_root(new_parent) {
            unwrap_or!(self.rename_tag(&previous_name, new_name),
                e, returne!(reply, EINVAL, error, "Failed to rename tag. {}",
                    e.to_string_wbt()));
            reply.ok();
            info!("Renamed tag.");
            return;
        }

        let all_namespaces = self.get_namespaces();
        let _previous_parent = all_namespaces.get_by_inode_id(previous_parent);
        let _new_parent = all_namespaces.get_by_inode_id(new_parent);
        if let (Ok(previous_parent), Ok(new_parent)) = (&_previous_parent, &_new_parent) {
            unwrap_or!(
                self.move_file(
                    &previous_parent.tags.clone(), &previous_name,
                    new_parent.tags.clone(), new_name),
                e, returne!(reply, EINVAL, error, "Failed to rename file. {}",
                    e.to_string_wbt()));
            reply.ok();
            info!("Renamed file.");
            return;
        }

        let mut e = format!("Renaming a tag, the parent has to be the root. \
            Renaming a file, the parent has to be a valid namespace. Previous \
            and new parent inodes are `{}` and `{}`.", previous_parent, new_parent);
        e.append_if_error(_previous_parent);
        e.append_if_error(_new_parent);
        error!(e);
        reply.error(EINVAL);
    }

    // TODO: Not confirmed to be implemented (pagination offset...), handle errors better
    // TODO: Does {e} get rendered?
    // TODO: set nowrap in nvim and reformat width of all codes
    #[instrument(skip_all, fields(?target_inode, ?start_position))]
    fn write(&mut self, _request: &Request<'_>, target_inode: u64, _file_handle: u64,
        start_position: i64, to_write: &[u8], write_flags: u32, flags: i32,
        _lock_owner: Option<u64>, reply: ReplyWrite)
    {
        let byte_amount: u32 = unwrap_or!(to_write.len().try_into().with_bt(),
            e, returne!(reply, EINVAL, error, "Writing too much data. {}", e.to_string_wbt()));

        let file_inode: FileInode = unwrap_or!(target_inode.try_into(),
            e, returne!(reply, EINVAL, error, "Is not a file inode. {}", e.to_string_wbt()));
        let start_position: u64 = unwrap_or!(start_position.try_into().with_bt(),
            e, returne!(reply, EINVAL, error, "Can't convert offset. {}", e.to_string_wbt()));
        unwrap_or!(self.write_to_file(&file_inode, start_position, to_write),
            e, returne!(reply, EINVAL, error, "{}", e.to_string_wbt()));

        reply.written(byte_amount);
        info!("Written.");
    }

    #[instrument(skip_all, fields(?target_inode))]
    fn setattr(&mut self, _request: &Request<'_>, target_inode: u64,
        _mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
        _atime: Option<TimeOrNow>, _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>,
        fh: Option<u64>, _crtime: Option<SystemTime>, _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>, flags: Option<u32>, reply: ReplyAttr)
    {
        // TODO
        if let Ok(attributes) = self.get_fuser_attributes(target_inode) {
            reply.attr(&ANY_TTL, &attributes);
            info!("Set attributes.");
        } else {
            error!("Inode does not match anything.");
            reply.error(ENOENT);
        }
    }

    #[instrument(skip_all, fields(?parent_inode, ?file_name))]
    fn unlink(&mut self, _request: &Request<'_>, parent_inode: u64,
        file_name: &OsStr, reply: ReplyEmpty)
    {
        if !get_is_inode_root(parent_inode) && !get_is_inode_a_namespace(parent_inode) {
            error!("Not child of TFS root nor a namespace.");
            reply.error(ENOENT);
            return;
        }

        let file_name = file_name.to_string_lossy();

        if get_is_inode_root(parent_inode) {
            unwrap_or!(
                self.remove_file_by_name_and_tags(
                    &file_name, &TagInodes::new()),
                e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        }

        if let Ok(parent_namespace) = self.get_namespaces()
            .get_by_inode_id(parent_inode)
        {
            unwrap_or!(
                self.remove_file_by_name_and_tags(
                    &file_name, &parent_namespace.tags.clone()),
                e, returne!(reply, ENOENT, error, "{}", e.to_string_wbt()));
        }

        reply.ok();
        info!("Deleted.");
    }

    #[instrument(skip_all, fields(?parent_inode, ?tag_name))]
    fn rmdir(&mut self, _request: &Request<'_>, parent_inode: u64,
        tag_name: &OsStr, reply: ReplyEmpty)
    {
        if !get_is_inode_root(parent_inode) {
            error!("Not child of TFS root.");
            reply.error(ENOENT);
            return;
        }
        
        if let Err(e) = self.delete_tag(&tag_name.to_string_lossy()) {
            returne!(reply, ENOENT, error, "Failed to delete tag. {}", e.to_string_wbt())
        }
        reply.ok();
        info!("Deleted.");
    }

    #[instrument(skip_all)]
    fn destroy(&mut self) {
        let max_tries = 4;
        let initial_cooldown = 1;
        for try_index in 0..max_tries {
            if let Err(e) = self.save_persistently() {
                error!("Failed to save TFS. {}", e.to_string_wbt());
            } else {
                break;
            }
            let is_last = try_index != max_tries;
            if is_last {
                let retry_cooldown = initial_cooldown << try_index;
                sleep(Duration::from_secs(retry_cooldown));
                info!("Slept `{}` seconds.", retry_cooldown); 
            }
        }
    }
}

// TODO: Give proper values.
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

fn get_is_a_namespace(value: &str) -> bool {
    value.chars().next() == Some('{')
}
