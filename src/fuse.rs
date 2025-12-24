use std::{ffi::OsStr, fmt::Display, thread::sleep, time::{Duration, SystemTime}};

use bon::Builder;
use derive_more::Error;
use fuser::{FileAttr, FileType, Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request, TimeOrNow, FUSE_ROOT_ID};
use libc::{c_int, EINVAL, ENOENT};
use tracing::{debug, error, info, instrument, trace, warn, Level};

use crate::{entries::TfsEntry, errors::{ResultBt, StringExt}, files::TfsFile,
    filesystem::TagFilesystem, inodes::{get_is_inode_root, FileInode, NamespaceInode, TagInode,
    TagInodes}, namespaces, os::{COMMON_BLOCK_SIZE, NO_RDEV, ROOT_GID, ROOT_UID},
    storage::TfsStorage, tags::{TfsTag, DEFAULT_TAG_PERMISSIONS}, ttl::{ANY_TTL, NO_TTL},
    ResultExt, ResultExt2};

macro_rules! event_ {
    ($tracing_level: expr, $error_message: expr, $($message_arguments: expr), *) => {{
        match $tracing_level {
            Level::TRACE => trace!($error_message, $($message_arguments), *),
            Level::DEBUG => debug!($error_message, $($message_arguments), *),
            Level::INFO  => info!($error_message, $($message_arguments), *),
            Level::WARN  => warn!($error_message, $($message_arguments), *),
            Level::ERROR => error!($error_message, $($message_arguments), *)
        }
    }};
}

macro_rules! handle_error_reply {
    ($fuser_reply: ident, $error_reply: ident) => {
        {
            event_!($error_reply.level, "{}", $error_reply.message);
            $fuser_reply.error($error_reply.code);
        }
    }
}

const ANY_GENERATION: u64 = 0;
const ANY_FILE_HANDLE: u64 = 0;
const ANY_FLAGS: u32 = 0;

fn get_is_a_namespace(value: &str) -> bool {
    value.chars().next() == Some('{')
}

// TODO(S):
// - Some `reply.error` should not really log as an error.
// - Sometimes the below error for `ct tag_2` when `tag2` does exist.
//   `Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }`
// - Should have two (or infinite) depth query sets? 
//   Cause like `{ tag_1, tag_2 }/file_1`, want to add `tag_3`, how to do with good ux?
//   cwd at `{ tag_1, tag_2 }`, `mv file_1 ./{ tag_3 }`,
//   and want to remove a tag, `mv file_1 ./{ ~tag_2 }`
// - Check they reply errors are the most suitable ones.
// - Errors need to be displayed to the user not just logged.
// - What does TTL, generation, fh, flags do?
// - Make some of the FUSE ops atomic
// - Get rid of `map_err`, make `map_err_inner`.
impl<Storage: TfsStorage> Filesystem for TagFilesystem<Storage> {
    #[instrument(skip_all, fields(?parent_inode, ?file_name))]
    fn create(&mut self, request: &Request<'_>, parent_inode: u64,
        file_name: &OsStr, mode: u32, umask: u32, flags: i32,
        reply: ReplyCreate)
    {
        match self.create_inner(request, parent_inode, file_name, mode, umask, flags) {
            Ok(_reply) => {
                reply.created(&_reply.ttl, &_reply.attr, _reply.generation, _reply.fh,
                    _reply.flags);
                info!("Created file.");
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?parent_inode, ?tag_name))]
    fn mkdir(&mut self, request: &Request<'_>, parent_inode: u64,
        tag_name: &OsStr, mode: u32, umask: u32, reply: ReplyEntry)
    {
        match self.mkdir_inner(request, parent_inode, tag_name, mode, umask) {
            Ok(_reply) => {
                reply.entry(&_reply.ttl, &_reply.attr, _reply.generation);
                info!("Created tag.");
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }
    
    #[instrument(skip_all, fields(?parent_inode, ?predicate))]
    fn lookup(&mut self, request: &Request, parent_inode: u64,
        predicate: &OsStr, reply: ReplyEntry)
    {
        match self.lookup_inner(request, parent_inode, predicate) {
            Ok(_reply) => {
                reply.entry(&_reply.ttl, &_reply.attr, _reply.generation);
                info!(_reply.message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?inode_id))]
    fn getattr(&mut self, request: &Request<'_>, inode_id: u64,
        file_handle: Option<u64>, reply: ReplyAttr)
    {
        match self.getattr_inner(request, inode_id, file_handle) {
            Ok(_reply) => {
                reply.attr(&_reply.ttl, &_reply.attr);
                info!(_reply.message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    // TODO(s): 
    // - Should probably have to -f when deleting tags w/ files under them.
    //   in `/tmp/tfs/` doing `rmdir tag_1` vs `rmdir "{ tag_1 }"
    // - Should allow listing of root or only allow {}?
    // - Determine if pagination can probably be race cond. in multi user
    #[instrument(skip_all, fields(?inode_id))]
    fn readdir(&mut self, request: &Request, inode_id: u64, file_handle: u64,
        pagination_offset: i64, mut reply: ReplyDirectory)
    {
        match self.readdir_inner(request, inode_id, file_handle, pagination_offset,
            &mut reply)
        {
            Ok(message) => {
                reply.ok();
                info!(message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?target_inode, ?start_position, ?read_amount))]
    fn read(&mut self, request: &Request<'_>, target_inode: u64, file_handle: u64,
        start_position: i64, read_amount: u32, flags: i32, lock_owner: Option<u64>,
        reply: ReplyData)
    {
        match self.read_inner(request, target_inode, file_handle,
            start_position, read_amount, flags, lock_owner)
        {
            Ok(_reply) => {
                reply.data(&_reply.data);
                info!(_reply.message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?target_inode))]
    fn fsyncdir(&mut self, request: &Request<'_>, target_inode: u64,
        file_handle: u64, datasync: bool, reply: ReplyEmpty)
    {
        match self.fsyncdir_inner(request, target_inode, file_handle,
            datasync)
        {
            Ok(message) => {
                reply.ok();
                info!(message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?previous_parent, ?previous_name, ?new_parent, ?new_name))]
    fn rename(&mut self, request: &Request<'_>, previous_parent: u64,
        previous_name: &OsStr, new_parent: u64, new_name: &OsStr, flags: u32,
        reply: ReplyEmpty)
    {
        match self.rename_inner(request, previous_parent, previous_name,
            new_parent, new_name, flags)
        {
            Ok(message) => {
                reply.ok();
                info!(message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    // TODO(s):
    // - Not confirmed to be implemented (pagination offset...), handle errors better
    // - Does {e} get rendered?
    #[instrument(skip_all, fields(?target_inode, ?start_position))]
    fn write(&mut self, request: &Request<'_>, target_inode: u64, file_handle: u64,
        start_position: i64, to_write: &[u8], write_flags: u32, flags: i32,
        lock_owner: Option<u64>, reply: ReplyWrite)
    {
        match self.write_inner(request, target_inode, file_handle, start_position,
            to_write, write_flags, flags, lock_owner)
        {
            Ok(_reply) => {
                reply.written(_reply.amount);
                info!(_reply.message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?target_inode))]
    fn setattr(&mut self, request: &Request<'_>, target_inode: u64,
        mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
        atime: Option<TimeOrNow>, mtime: Option<TimeOrNow>, ctime: Option<SystemTime>,
        fh: Option<u64>, crtime: Option<SystemTime>, chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>, flags: Option<u32>, reply: ReplyAttr)
    {
        match self.setattr_inner(request, target_inode, mode, uid, gid, size,
            atime, mtime, ctime, fh, crtime, chgtime, bkuptime, flags)
        {
            Ok(_reply) => {
                reply.attr(&_reply.ttl, &_reply.attr);
                info!(_reply.message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?parent_inode, ?file_name))]
    fn unlink(&mut self, request: &Request<'_>, parent_inode: u64, file_name: &OsStr,
        reply: ReplyEmpty)
    {
        match self.unlink_inner(request, parent_inode, file_name) {
            Ok(message) => {
                reply.ok();
                info!(message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
    }

    #[instrument(skip_all, fields(?parent_inode, ?tag_name))]
    fn rmdir(&mut self, request: &Request<'_>, parent_inode: u64, tag_name: &OsStr,
        reply: ReplyEmpty)
    {
        match self.rmdir_inner(request, parent_inode, tag_name) {
            Ok(message) => {
                reply.ok();
                info!(message);
            },
            Err(_reply) => handle_error_reply!(reply, _reply)
        }
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

#[derive(Debug, Error, Builder)]
#[builder(on(String, into))]
struct ErrorReply {
    code: c_int,
    #[builder(default = Level::ERROR)]
    level: Level,
    message: String
}

impl Display for ErrorReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` (errno `{}`).", self.message, self.code)
    }
}

// TODO: How to further minimize error returning propagation?
// Could do trait ext. on Result 
//
// I wonder if you can make it implicitly/default to a specific return value
// impl From<crate::WithBacktrace<crate::errors::AnyError>> for crate::WithBacktrace<ErrorReply> {
//     fn from(value: crate::WithBacktrace<crate::errors::AnyError>) -> Self {
//         todo!()
//     }
// }
impl ErrorReply {
    fn new(code: c_int, message: impl Into<String>) -> Self {
        Self {
            code,
            level: Level::ERROR,
            message: message.into()
        }
    }

    fn new_with_level(code: c_int, level: Level, message: impl Into<String>) -> Self {
        Self {
            code,
            level,
            message: message.into()
        }
    }
}

struct CreateReply {
    ttl: Duration,
    attr: FileAttr,
    generation: u64,
    fh: u64,
    flags: u32,
}

struct MkdirReply {
    ttl: Duration,
    attr: FileAttr,
    generation: u64,
}

struct LookupReply {
    ttl: Duration,
    attr: FileAttr,
    generation: u64,
    message: String
}

struct GetattrReply {
    ttl: Duration,
    attr: FileAttr,
    message: &'static str 
}

struct DataReply {
    data: Vec<u8>,
    message: &'static str
}

struct WriteReply {
    amount: u32,
    message: &'static str
}

struct SetattrReply {
    ttl: Duration,
    attr: FileAttr,
    message: &'static str
}

impl<Storage: TfsStorage> TagFilesystem<Storage> {
    // TODO(s):
    // - Use rest of args, or at least understand them.
    // - Remove _ prefix if used
    fn create_inner(&mut self, request: &Request<'_>, parent_inode: u64,
        file_name: &OsStr, _mode: u32, umask: u32, _flags: i32)
        -> ResultBt<CreateReply, ErrorReply>
    {
        if !get_is_inode_root(parent_inode) && !NamespaceInode::get_is_namespace(parent_inode) {
            Err(ErrorReply::new(ENOENT, "Not child of TFS root nor a namespace."))?;
        }

        let new_file = TfsFile::builder()
            .name(file_name.to_string_lossy().clone())
            .inode(self.get_free_file_inode()
                .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?)
            .owner(request.uid())
            .group(request.gid())
            .permissions(DEFAULT_TAG_PERMISSIONS & !(umask as u16));

        if get_is_inode_root(parent_inode) {
            let new_file = self.add_file(new_file.build())
                .map_err_inner(|e| ErrorReply::new(EINVAL, e.to_string()))?;
            // TODO: I swear this should not be needed : \
            let file_inode = new_file.inode;
            let fuser_attributes = self.get_file_fuser(&file_inode)
                .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
            return Ok(CreateReply {
                ttl: ANY_TTL,
                attr: fuser_attributes,
                generation: ANY_GENERATION,
                fh: ANY_FILE_HANDLE,
                flags: ANY_FLAGS
            });
        }

        let namespace_inode = NamespaceInode::try_from(parent_inode)
            .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        let tfs_namespace = self.get_namespaces()
            .get_map()
            .get(&namespace_inode)
            .ok_or(ErrorReply::new(ENOENT, format!("Namespace with id \
                `{namespace_inode}` does not exist.")))?;

        let new_file = self.add_file(new_file.tags(tfs_namespace.tags.clone()).build())
            .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        let file_inode = new_file.inode;
        let fuser_attributes = self.get_file_fuser(&file_inode)
            // TODO: More appropriate error code.
            .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        Ok(CreateReply {
            ttl: ANY_TTL,
            attr: fuser_attributes,
            generation: ANY_GENERATION,
            fh: ANY_FILE_HANDLE,
            flags: ANY_FLAGS
        })
    }

    // TODO: Figure out eval steps. for files and tags inheriting perms (setgid).
    fn mkdir_inner(&mut self, request: &Request<'_>, parent_inode: u64,
        tag_name: &OsStr, _mode: u32, umask: u32)
        -> ResultBt<MkdirReply, ErrorReply>
    {
        let tag_name = tag_name.to_string_lossy();

        // TODO: Maybe allow mkdir everywhere, just always create at global.
        if !get_is_inode_root(parent_inode) {
            Err(ErrorReply::new(ENOENT, "Needs to be under the root directory."))?;
        }

        let is_file_conflicting = self.get_files()
            .get_by_name_and_tags(&tag_name, &TagInodes::new())
            .is_some();
        if is_file_conflicting {
            Err(ErrorReply::new(EINVAL, format!("Untagged file already exists with \
                name `{tag_name}`.")))?;
        }

        let new_tag = self.add_tag()
            .tag_name(tag_name)
            .owner_id(request.uid())
            .group_id(request.gid())
            .permissions(DEFAULT_TAG_PERMISSIONS & !(umask as u16))
            .call()
            .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        let tag_inode = new_tag.inode;
        let fuser_attributes = self.get_tag_fuser(&tag_inode)
            .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        Ok(MkdirReply {
            ttl: ANY_TTL,
            attr: fuser_attributes,
            generation: ANY_GENERATION
        })
    }

    fn lookup_inner(&mut self, request: &Request, parent_inode: u64,
        predicate: &OsStr) -> ResultBt<LookupReply, ErrorReply>
    {
        // TODO: Is there not just a method that returns String instead of Cow?
        let predicate = predicate.to_string_lossy().to_string();

        if get_is_inode_root(parent_inode) {
            if get_is_a_namespace(&predicate) {
                let namespace_inode = self.add_namespace_with_name()
                    .namespace_string(predicate)
                    .owner_id(request.uid())
                    .group_id(request.gid())
                    .call()
                    .map_err_inner(|e| ErrorReply::new(ENOENT,
                        format!("Namespace lookup failed. {}", e.to_string())))?
                    .inode;
                return Ok(LookupReply {
                    ttl: NO_TTL,
                    attr: namespaces::get_fuse_attributes(&namespace_inode),
                    generation: ANY_GENERATION,
                    message: String::from("Completed namespace lookup.")
                });
            }

            // TODO: Can this be put in function to have single source of truth?
            let target_inode = self.get_tags()
                .get_by_name(&predicate)
                .map(|tag| tag.inode.get_id())
                .or(self.get_files()
                    .get_by_name_and_tags(&predicate, &TagInodes::new())
                    .map(|file| file.inode.get_id()))
                .ok_or(ErrorReply::new_with_level(ENOENT, Level::INFO, format!(
                    "Tag/file lookup failed, `{predicate}` is not a tag nor an \
                    untagged file.")))?;

            let fuser_attributes = self.get_fuser_attributes(target_inode)
                .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
            // TODO: See if setting query to None after this is appropriate.
            return Ok(LookupReply {
                ttl: ANY_TTL,
                attr: fuser_attributes,
                generation: ANY_GENERATION,
                message: String::from("Completed tag/file lookup under root.")
            });
        }

        // TODO: Need to add checks as to what file and tag names can be created
        // to make them not conflict, no dupes when ls'ing
        if let Ok(parent_namespace) = self.get_namespaces().get_by_inode_id(parent_inode) {
            // TODO: Can this be put in function with twin-ish
            let fuser = self.get_inrange_tags(parent_namespace)
                .and_then(|tags| tags.into_iter()
                    .find(|tag| tag.name == predicate)
                    .ok_or(format!("No matching neighbour tag w/ name `{}` \
                        for namespace `{}`.", predicate, parent_namespace.inode).into()))
                .and_then(|tag| self.get_tag_fuser(&tag.inode))
                .or(self.get_file_by_name_and_namespace_inode(&predicate, &parent_namespace.inode)
                    .and_then(|file| self.get_file_fuser(&file.inode)))
                .map_err_inner(|e| ErrorReply::new(ENOENT, format!("Tag/file \
                    lookup failed, `{predicate}` is not a tag not a file under \
                    inode `{parent_inode}`. {}", e.to_string())))?;
            // TODO: no magic variables.
            return Ok(LookupReply {
                ttl: NO_TTL,
                attr: fuser,
                generation: ANY_GENERATION,
                message: String::from("Completed tag/file lookup under namespace.")
            });
        } 

        Err(ErrorReply::new(ENOENT, format!("No tags or files matching \
            `{predicate}` under root and/or namespace, `{parent_inode}`.")))?
    }

    fn getattr_inner(&mut self, _request: &Request<'_>, inode_id: u64,
        _file_handle: Option<u64>) -> ResultBt<GetattrReply, ErrorReply>
    {
        // TODO: See impact of NO_TTL
        // TODO: Use get_is_inode_root
        if get_is_inode_root(inode_id) {
            return Ok(GetattrReply {
                ttl: NO_TTL,
                attr: self.get_root_fuser(),
                message: "Replied w/ root."
            });
        }
        
        if let Ok(namespace_inode) = NamespaceInode::try_from(inode_id) {
            return Ok(GetattrReply {
                ttl: NO_TTL,
                attr: namespaces::get_fuse_attributes(&namespace_inode),
                message: "Replied w/ namespace."
            });
        }

        if let Ok(fuser_attributes) = FileInode::try_from(inode_id)
            .and_then(|inode| self.get_file_fuser(&inode))
            .or(TagInode::try_from(inode_id)
                .and_then(|inode| self.get_tag_fuser(&inode))) {
                    return Ok(GetattrReply {
                        ttl: ANY_TTL,
                        attr: fuser_attributes,
                        message: "Replied w/ file."
                    });
                }

        Err(ErrorReply::new(ENOENT, "Did not match any inode."))?
    }

    fn readdir_inner(&mut self, _request: &Request, inode_id: u64, _file_handle: u64,
        mut pagination_offset: i64, mut reply: &mut ReplyDirectory)
        -> ResultBt<&'static str, ErrorReply>
    {
        let is_listing_root = get_is_inode_root(inode_id);
        if !is_listing_root && !NamespaceInode::get_is_namespace(inode_id) {
            Err(ErrorReply::new(ENOENT, "Inode not root or a namespace."))?;
        }

        let pagination_offset_: usize = pagination_offset.try_into().with_bt()
                .map_err_inner(|e| ErrorReply::new(
                    EINVAL, format!("Can't convert offset. {e}")))?;

        let add = |tfs_entry: &dyn TfsEntry, pagination_offset: i64, reply: &mut ReplyDirectory| {
            reply.add(tfs_entry.get_inode_id(), pagination_offset,
                tfs_entry.get_file_kind(), tfs_entry.get_name())
        };

        if is_listing_root {
            let mut tagless_files: Vec<_> = self.get_files()
                .get_by_tags(&TagInodes::new())
                .collect();
            tagless_files.sort();
            let tagless_files = tagless_files.into_iter().map(|file| file as &dyn TfsEntry);

            let mut all_tags: Vec<_> = self.get_tags().get_all().collect();
            all_tags.sort();
            let all_tags = all_tags.into_iter().map(|tag| tag as &dyn TfsEntry);

            for tfs_entry in tagless_files.chain(all_tags).skip(pagination_offset_) {
                pagination_offset += 1;
                let had_space = add(tfs_entry, pagination_offset, &mut reply);
                if !had_space {
                    return Ok("Partially listed root.");
                }
            }

            return Ok("Finished listing root.");
        }

        let current_namespace = self.get_namespaces().get_by_inode_id(inode_id)
            .map_err_inner(|e| ErrorReply::new(EINVAL,  format!("Could not get \
                namespace. {e}")))?;

        let mut inrange_tags = self.get_inrange_tags(current_namespace)
            .map_err_inner(|e| ErrorReply::new(EINVAL, format!("Could not get \
                inrange tags. {e}")))?;
        inrange_tags.sort();
        let inrange_tags = inrange_tags.into_iter()
            .map(|tag| tag as &dyn TfsEntry);

        let mut inscope_files: Vec<_> = self.get_files_by_namespace_inode(
            &current_namespace.inode)
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Could not get files under namespace. {e}")))?
            .collect();
        inscope_files.sort();
        let inscope_files = inscope_files.into_iter()
            .map(|file| file as &dyn TfsEntry);

        for to_list in inscope_files.chain(inrange_tags)
            .skip(pagination_offset_)
        {
            pagination_offset += 1;
            let had_space = add(to_list, pagination_offset, &mut reply);
            if !had_space {
                return Ok("Partially finished listing namespace.");
            }
        }

        Ok("Finished finished listing namespace.")
    }

    fn read_inner(&mut self, _request: &Request<'_>, target_inode: u64, _file_handle: u64,
        start_position: i64, read_amount: u32, flags: i32, _lock_owner: Option<u64>)
        -> ResultBt<DataReply, ErrorReply>
    {
        let file_inode: FileInode = target_inode.try_into()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Not a file inode. {e}")))?; 
        let start_position: u64 = start_position.try_into().with_bt()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Offset value can't be converted. {e}")))?; 
        let read_amount: usize = read_amount.try_into().with_bt()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Amount to read can't be converted. {e}")))?;

        let content_read = self.get_storage()
            .read(&file_inode, start_position, read_amount)
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Failed to read file. {e}")))?;

        Ok(DataReply {
            data: content_read,
            message: "Read completed."
        })
    }

    fn fsyncdir_inner(&mut self, _request: &Request<'_>, target_inode: u64,
        _file_handle: u64, _datasync: bool) -> ResultBt<&'static str, ErrorReply>
    {
        let should_fsync_all = get_is_inode_root(target_inode); 
        if should_fsync_all {
            self.save_persistently()
                .map_err_inner(|e| ErrorReply::new(
                    EINVAL, format!("Failed to save TFS state. {e}")))?;
            return Ok("Saved all.");
        }

        // TODO
        Err(ErrorReply::new(EINVAL, "Not implemented yet."))?
    }

    fn rename_inner(&mut self, _request: &Request<'_>, previous_parent: u64,
        previous_name: &OsStr, new_parent: u64, new_name: &OsStr, _flags: u32)
        -> ResultBt<&'static str, ErrorReply>
    {
        let previous_name = previous_name.to_string_lossy();
        let new_name = new_name.to_string_lossy().to_string();

        if get_is_inode_root(previous_parent) && get_is_inode_root(new_parent) {
            self.rename_tag(&previous_name, new_name)
                .map_err_inner(|e| ErrorReply::new(
                    EINVAL, format!("Failed to rename tag. {e}")))?;
            return Ok("Renamed tag.");
        }

        let all_namespaces = self.get_namespaces();
        let _previous_parent = all_namespaces.get_by_inode_id(previous_parent);
        let _new_parent = all_namespaces.get_by_inode_id(new_parent);
        if let (Ok(previous_parent), Ok(new_parent)) = (&_previous_parent, &_new_parent) {
            self.move_file(
                &previous_parent.tags.clone(), &previous_name,
                new_parent.tags.clone(), new_name)
                .map_err_inner(|e| ErrorReply::new(
                    EINVAL, format!("Failed to rename file. {e}")))?;
            return Ok("Renamed file.");
        }

        let mut e = format!("Renaming a tag, the parent has to be the root. \
            Renaming a file, the parent has to be a valid namespace. Previous \
            and new parent inodes are `{}` and `{}`.", previous_parent, new_parent);
        e.append_if_error(_previous_parent);
        e.append_if_error(_new_parent);
        Err(ErrorReply::new(EINVAL, e))?
    }

    fn write_inner(&mut self, _request: &Request<'_>, target_inode: u64,
        _file_handle: u64, start_position: i64, to_write: &[u8], write_flags: u32,
        flags: i32, _lock_owner: Option<u64>) -> ResultBt<WriteReply, ErrorReply>
    {
        let byte_amount: u32 = to_write.len().try_into().with_bt()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Writing too much data. {e}")))?;

        let file_inode: FileInode = target_inode.try_into()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Is not a file inode. {e}")))?;
        let start_position: u64 = start_position.try_into().with_bt()
            .map_err_inner(|e| ErrorReply::new(
                EINVAL, format!("Can't convert offset. {e}")))?;
        self.write_to_file(&file_inode, start_position, to_write)
            .map_err_inner(|e| ErrorReply::new(EINVAL, e.to_string()))?;

        Ok(WriteReply {
            amount: byte_amount,
            message: "Written."
        })
    }

    fn setattr_inner(&mut self, _request: &Request<'_>, target_inode: u64,
        _mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
        _atime: Option<TimeOrNow>, _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>,
        fh: Option<u64>, _crtime: Option<SystemTime>, _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>, flags: Option<u32>)
        -> ResultBt<SetattrReply, ErrorReply>
    {
        let fuser_attributes = self.get_fuser_attributes(target_inode)
            .map_err_inner(|e| ErrorReply::new(
                ENOENT, format!("Inode does not match anything. {e}")))?;

        // TODO
        Ok(SetattrReply {
            ttl: ANY_TTL,
            attr: fuser_attributes,
            message: "Set attributes."
        })
    }

    fn unlink_inner(&mut self, _request: &Request<'_>, parent_inode: u64,
        file_name: &OsStr) -> ResultBt<&'static str, ErrorReply>
    {
        if !get_is_inode_root(parent_inode) && !NamespaceInode::get_is_namespace(parent_inode) {
            Err(ErrorReply::new(ENOENT, String::from("Not child of TFS root nor \
                a namespace.")))?;
        }

        let file_name = file_name.to_string_lossy();

        if get_is_inode_root(parent_inode) {
            self.remove_file_by_name_and_tags(&file_name, &TagInodes::new())
                .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        }

        if let Ok(parent_namespace) = self.get_namespaces().get_by_inode_id(parent_inode) {
            self.remove_file_by_name_and_tags( &file_name, &parent_namespace.tags.clone())
                .map_err_inner(|e| ErrorReply::new(ENOENT, e.to_string()))?;
        }

        Ok("Deleted.")
    }

    fn rmdir_inner(&mut self, _request: &Request<'_>, parent_inode: u64,
        tag_name: &OsStr) -> ResultBt<&'static str, ErrorReply>
    {
        if !get_is_inode_root(parent_inode) {
            Err(ErrorReply::new(ENOENT, "Not child of TFS root."))?
        }
        
        self.delete_tag(&tag_name.to_string_lossy())
            .map_err_inner(|e| ErrorReply::new(
                ENOENT, format!("Failed to delete tag. {e}")))?;

        Ok("Deleted.")
    }
}
