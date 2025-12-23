use std::{io::{BufRead, Write}, time::{Duration, SystemTime, UNIX_EPOCH}};

use bon::builder;
use capnp::{message::{self, ReaderOptions}, serialize_packed};
use fuser::{FileAttr, FileType, FUSE_ROOT_ID};

use crate::{coalesce, coalescerr, errors::{AnyError, ResultBtAny}, files::TfsFile, filesystem_capnp::tag_filesystem, inodes::{FileInode, TagInode}, os::{COMMON_BLOCK_SIZE, NO_RDEV}, tags::TfsTag, ResultExt};

macro_rules! get_system_times {
    ($capnp_reader: ident) => {
        (
            new_unix_epoch(
                $capnp_reader.get_when_accessed_seconds(),
                $capnp_reader.get_when_accessed_nanoseconds())
                .map_err_inner(|e| format!("When accessed failed. {e}")),
            new_unix_epoch(
                $capnp_reader.get_when_modified_seconds(),
                $capnp_reader.get_when_modified_nanoseconds())
                .map_err_inner(|e| format!("When modified failed. {e}")),
            new_unix_epoch(
                $capnp_reader.get_when_changed_seconds(),
                $capnp_reader.get_when_changed_nanoseconds())
                .map_err_inner(|e| format!("When changed failed. {e}")),
            new_unix_epoch(
                $capnp_reader.get_when_created_seconds(),
                $capnp_reader.get_when_created_nanoseconds())
                .map_err_inner(|e| format!("When created failed. {e}"))
        )
    }
}

#[builder]
pub fn new_root_fuser(uid: u32, gid: u32, permissions: u16, when_accessed: SystemTime,
    when_modified: SystemTime, when_changed: SystemTime, when_created: SystemTime)
    -> FileAttr
{
    // TODO/WIP: Give proper values.
    FileAttr {
        ino: FUSE_ROOT_ID,
        size: 0,
        blocks: 0,
        atime: when_accessed,
        mtime: when_modified,
        ctime: when_changed,
        crtime: when_created,
        kind: FileType::Directory,
        perm: permissions,
        nlink: 0,
        uid,
        gid,
        rdev: NO_RDEV,
        flags: 0,
        blksize: COMMON_BLOCK_SIZE,
    }
}

pub struct PersistedTfs {
    pub root: FileAttr,
    pub files: Vec<TfsFile>,
    pub tags: Vec<TfsTag>
}

pub fn deserialize_tag_filesystem(read_location: impl BufRead)
    -> ResultBtAny<PersistedTfs>
{
    let capnp_message = serialize_packed::read_message(read_location,
        ReaderOptions::new())?;
    let capnp_filesystem = capnp_message.get_root::<tag_filesystem::Reader>()?;

    // TODO/WIP: Get rid of map_err, make map_err_inner.
    let root_fuser = capnp_filesystem.get_root()?;
    let (when_accessed, when_modified, when_changed, when_created)
        = get_system_times!(root_fuser);
    let (when_accessed, when_modified, when_changed, when_created) = coalesce!(
        "For root FUSE attributes not all fields could be deserialized.",
        when_accessed, when_modified, when_changed, when_created)?;
    let _root_fuser = new_root_fuser()
        .uid(root_fuser.get_owner())
        .gid(root_fuser.get_group())
        .permissions(root_fuser.get_permissions())
        .when_accessed(when_accessed)
        .when_modified(when_modified)
        .when_changed(when_changed)
        .when_created(when_created)
        .call();

    let mut tfs_files = vec![];
    for capnp_file in capnp_filesystem.get_files()? {
        let file_name = capnp_file.get_name()
            .map_err(AnyError::from)
            .and_then(|name| name.to_string()
                .map_err(AnyError::from));
        let file_inode = FileInode::try_from(capnp_file.get_inode());
        let (when_accessed, when_modified, when_changed, when_created)
            = get_system_times!(capnp_file);
        let tag_inodes = capnp_file.get_tags()
            .map_err(AnyError::from)
            .and_then(|inodes| {
                let mut _inodes = vec![];
                let mut errors = vec![];
                for tag_inode in inodes {
                    match TagInode::try_from(tag_inode) {
                        Ok(inode) => _inodes.push(inode),
                        Err(e) => errors.push(e),
                    }
                }
                if !errors.is_empty() {
                    return Err(errors.iter()
                        .map(|e| e.to_string_wbt())
                        .collect::<Vec<_>>()
                        .join(". ")
                        .into());
                }
                Ok(_inodes.into_iter())
            });

        let (file_name, file_inode,
            when_accessed, when_modified, when_changed, when_created,
            tag_inodes)
            = coalesce!("Not all file fields could be deserialized.",
            file_name, file_inode,
            when_accessed, when_modified, when_changed, when_created,
            tag_inodes)?;
        
        tfs_files.push(TfsFile { 
            name: file_name,
            inode: file_inode,
            owner: capnp_file.get_owner(),
            group: capnp_file.get_group(),
            permissions: capnp_file.get_permissions(),
            when_accessed,
            when_modified,
            when_changed,
            when_created,
            tags: tag_inodes.into(),
        });
    }

    let mut tfs_tags = vec![];
    for capnp_tag in capnp_filesystem.get_tags()? {
        let tag_name = capnp_tag.get_name()
            .map_err(AnyError::from)
            .and_then(|name| name.to_string()
                .map_err(AnyError::from));
        let tag_inode = TagInode::try_from(capnp_tag.get_inode());
        let (when_accessed, when_modified, when_changed, when_created)
            = get_system_times!(capnp_tag);

        let (tag_name, tag_inode,
            when_accessed, when_modified, when_changed, when_created)
            = coalesce!("Not all tag fields could be deserialized.",
            tag_name, tag_inode,
            when_accessed, when_modified, when_changed, when_created)?;

        tfs_tags.push(TfsTag { 
            name: tag_name,
            inode: tag_inode,
            owner: capnp_tag.get_owner(),
            group: capnp_tag.get_group(),
            permissions: capnp_tag.get_permissions(),
            when_accessed,
            when_modified,
            when_changed,
            when_created
        });
    }

    Ok(PersistedTfs {
        root: _root_fuser,
        files: tfs_files,
        tags: tfs_tags
    })
}

pub fn serialize_tag_filesystem(write_location: impl Write, root_fuser: &FileAttr,
    tfs_files: Vec<&TfsFile>, tfs_tags: Vec<&TfsTag>) -> ResultBtAny<()>
{
    type CapnpType = u32;

    let mut capnp_message = message::Builder::new_default();
    let mut capnp_filesystem = capnp_message.init_root::<tag_filesystem::Builder>();

    let mut _root_fuser = capnp_filesystem.reborrow().init_root();
    _root_fuser.set_owner(root_fuser.uid);
    _root_fuser.set_group(root_fuser.gid);
    _root_fuser.set_permissions(root_fuser.perm);
    let when_accessed = root_fuser.atime.duration_since(UNIX_EPOCH);
    let when_modified = root_fuser.mtime.duration_since(UNIX_EPOCH);
    let when_changed = root_fuser.ctime.duration_since(UNIX_EPOCH);
    let when_created = root_fuser.crtime.duration_since(UNIX_EPOCH);
    coalescerr!("For root FUSE attributes not all fields could be serialized.", when_accessed, when_modified, when_changed, when_created);
    _root_fuser.set_when_accessed_seconds(when_accessed.as_secs());
    _root_fuser.set_when_modified_seconds(when_modified.as_secs());
    _root_fuser.set_when_changed_seconds(when_changed.as_secs());
    _root_fuser.set_when_created_seconds(when_created.as_secs());

    let file_count = tfs_files.len();
    let file_count = CapnpType::try_from(file_count)
        .map_err(|e| format!("Cannot convert number of files `{}` to Cap'n Proto \
            length type. {e}", file_count))?;
    let mut capnp_files = capnp_filesystem.reborrow().init_files(file_count);
    for (file_index, tfs_file) in tfs_files.iter().enumerate() {
        let file_index = CapnpType::try_from(file_index)?;
        let when_accessed = tfs_file.when_accessed.duration_since(UNIX_EPOCH);
        let when_modified = tfs_file.when_modified.duration_since(UNIX_EPOCH);
        let when_changed = tfs_file.when_changed.duration_since(UNIX_EPOCH);

        let file_tags = &tfs_file.tags.0;
        let tags_count = CapnpType::try_from(file_tags.len());

        match (when_accessed, when_modified, when_changed, tags_count) {
            (Ok(accessed), Ok(modified), Ok(changed), Ok(tags_count)) => {
                let mut capnp_file = capnp_files.reborrow().get(file_index);
                capnp_file.set_name(tfs_file.name.clone());
                capnp_file.set_inode(tfs_file.inode.get_id());
                capnp_file.set_owner(tfs_file.owner);
                capnp_file.set_group(tfs_file.group);
                capnp_file.set_permissions(tfs_file.permissions);
                capnp_file.set_when_accessed_seconds(accessed.as_secs());
                capnp_file.set_when_modified_seconds(modified.as_secs());
                capnp_file.set_when_changed_seconds(changed.as_secs());
                let mut capnp_tags = capnp_file.init_tags(tags_count);
                for (tag_index, file_tag) in file_tags.iter().enumerate() {
                    capnp_tags.set(CapnpType::try_from(tag_index)?, file_tag.get_id());
                } 
            },
            (accessed, modified, changed, tags_count) => {
                return Err(format!("For file with name `{}` and inode `{}`, \
                    not all fields could be serialized: \
                    accessed `{accessed:?}`, modified `{modified:?}`, \
                    changed `{changed:?}`, tags count `{tags_count:?}.",
                    tfs_file.name, tfs_file.inode).into());
            }
        }
    }

    let tag_count = tfs_tags.len();
    let tag_count = CapnpType::try_from(tag_count)
        .map_err(|e| format!("Cannot convert number of tags `{}` to Cap'n Proto \
            length type. {e}", file_count))?;
    let mut capnp_tags = capnp_filesystem.reborrow().init_tags(tag_count);
    for (tag_index, tfs_tag) in tfs_tags.iter().enumerate() {
        let tag_index = CapnpType::try_from(tag_index)?;

        let when_accessed = tfs_tag.when_accessed.duration_since(UNIX_EPOCH);
        let when_modified = tfs_tag.when_modified.duration_since(UNIX_EPOCH);
        let when_changed = tfs_tag.when_changed.duration_since(UNIX_EPOCH);

        match (when_accessed, when_modified, when_changed) {
            (Ok(accessed), Ok(modified), Ok(changed)) => {
                let mut capnp_tag = capnp_tags.reborrow().get(tag_index);
                capnp_tag.set_name(tfs_tag.name.clone());
                capnp_tag.set_inode(tfs_tag.inode.get_id());
                capnp_tag.set_owner(tfs_tag.owner);
                capnp_tag.set_group(tfs_tag.group);
                capnp_tag.set_permissions(tfs_tag.permissions);
                capnp_tag.set_when_accessed_seconds(accessed.as_secs());
                capnp_tag.set_when_modified_seconds(modified.as_secs());
                capnp_tag.set_when_changed_seconds(changed.as_secs());
            },
            (accessed, modified, changed) => {
                return Err(format!("For tag with name `{}` and inode `{}`, \
                    not all fields could be serialized: \
                    accessed `{accessed:?}`, modified `{modified:?}`, \
                    changed `{changed:?}`.",
                    tfs_tag.name, tfs_tag.inode).into());
            }
        }
    }

    serialize_packed::write_message(write_location, &capnp_message)?;

    Ok(())
}

// TODO/WIP: Use nanos.
#[deprecated]
fn as_system_time_unix_epoch(unix_epoch: u64) -> ResultBtAny<SystemTime> {
    UNIX_EPOCH.checked_add(
        Duration::from_secs(unix_epoch))
        .ok_or(format!("Invalid Unix epoch, `{}`.", unix_epoch).into())
}

fn new_unix_epoch(seconds: u64, nanoseconds: u32) -> ResultBtAny<SystemTime> {
    UNIX_EPOCH.checked_add(
        Duration::new(seconds, nanoseconds))
        .ok_or(format!("Overflowed `SystemTime`, `{seconds}` seconds with \
            `{nanoseconds}` nanoseconds.").into())
}
