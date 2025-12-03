use std::{io::{BufRead, Cursor, Write}, time::{Duration, SystemTime, UNIX_EPOCH}};

use capnp::{message::{self, ReaderOptions}, serialize_packed};

use crate::{errors::{AnyError, Result_}, files::TfsFile, filesystem_capnp::tag_filesystem, inodes::{FileInode, TagInode, TagInodes}, tags::TfsTag};

pub fn deserialize_tag_filesystem(read_location: impl BufRead)
    -> Result_<(Vec<TfsFile>, Vec<TfsTag>)>
{
    let capnp_message = serialize_packed::read_message(read_location,
        ReaderOptions::new())?;
    let capnp_filesystem = capnp_message.get_root::<tag_filesystem::Reader>()?;

    let mut tfs_files = vec![];
    for capnp_file in capnp_filesystem.get_files()? {
        let file_name = capnp_file.get_name()
            .map_err(AnyError::from)
            .and_then(|name| name.to_string()
                .map_err(AnyError::from));
        let file_inode = FileInode::try_from(capnp_file.get_inode());
        let when_accessed = as_system_time_unix_epoch(capnp_file.get_when_accessed());
        let when_modified = as_system_time_unix_epoch(capnp_file.get_when_modified());
        let when_changed = as_system_time_unix_epoch(capnp_file.get_when_changed());
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
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(". ")
                        .into());
                }
                Ok(_inodes.into_iter())
            });
        
        match (
            file_name, file_inode, when_accessed,
            when_modified, when_changed, tag_inodes
        ) {
            (
                Ok(name), Ok(inode), Ok(accessed),
                Ok(modified), Ok(changed), Ok(tags)
            ) => {
                tfs_files.push(TfsFile { 
                    name,
                    inode,
                    owner: capnp_file.get_owner(),
                    group: capnp_file.get_group(),
                    permissions: capnp_file.get_permissions(),
                    when_accessed: accessed,
                    when_modified: modified,
                    when_changed: changed,
                    tags: tags.into()
                });
            },
            (name, inode, accessed, modified, changed, tags) => {
                return Err(format!("Not all file fields could be deserialized: \
                    name `{name:?}`, inode `{inode:?}`, accessed `{accessed:?}`, \
                    modified `{modified:?}`, changed `{changed:?}`,
                    tags `{tags:?}`.").into());
            }
        }
    }

    let mut tfs_tags = vec![];
    for capnp_tag in capnp_filesystem.get_tags()? {
        let tag_name = capnp_tag.get_name()
            .map_err(AnyError::from)
            .and_then(|name| name.to_string()
                .map_err(AnyError::from));
        let tag_inode = TagInode::try_from(capnp_tag.get_inode());
        let when_accessed = as_system_time_unix_epoch(capnp_tag.get_when_accessed());
        let when_modified = as_system_time_unix_epoch(capnp_tag.get_when_modified());
        let when_changed = as_system_time_unix_epoch(capnp_tag.get_when_changed());
        match (
            tag_name, tag_inode, when_accessed,
            when_modified, when_changed
        ) {
            (
                Ok(name), Ok(inode), Ok(accessed),
                Ok(modified), Ok(changed)
            ) => {
                tfs_tags.push(TfsTag { 
                    name,
                    inode,
                    owner: capnp_tag.get_owner(),
                    group: capnp_tag.get_group(),
                    permissions: capnp_tag.get_permissions(),
                    when_accessed: accessed,
                    when_modified: modified,
                    when_changed: changed
                });
            },
            (name, inode, accessed, modified, changed) => {
                return Err(format!("Not all tag fields could be deserialized: \
                    name `{name:?}`, inode `{inode:?}`, accessed `{accessed:?}`, \
                    modified `{modified:?}`, changed `{changed:?}`.").into());
            }
        }
    }

    Ok((tfs_files, tfs_tags))
}

fn as_system_time_unix_epoch(unix_epoch: u64) -> Result_<SystemTime> {
    UNIX_EPOCH.checked_add(
        Duration::from_secs(unix_epoch))
        .ok_or(format!("Invalid Unix epoch, `{}`.", unix_epoch).into())
}

// TODO/WIP
pub fn serialize_tag_filesystem(write_location: &impl Write,
    tfs_files: Vec<&TfsFile>, tfs_tags: Vec<&TfsTag>)
    -> Result_<()>
{
    let mut capnp_message = message::Builder::new_default();
    let mut capnp_filesystem = capnp_message.init_root::<tag_filesystem::Builder>();

    let mut capnp_files = capnp_filesystem.reborrow().init_files(tfs_files.len().try_into()?);
    for (file_index, tfs_file) in tfs_files.iter().enumerate() {
        let mut capnp_file = capnp_files.reborrow().get(file_index.try_into()?);
        capnp_file.set_name(tfs_file.name.clone());
        capnp_file.set_inode(tfs_file.inode.get_id());
        capnp_file.set_owner(tfs_file.owner);
        capnp_file.set_group(tfs_file.group);
        capnp_file.set_permissions(tfs_file.permissions);
        capnp_file.set_when_accessed(tfs_file.when_accessed.duration_since(UNIX_EPOCH)?.as_secs());
        capnp_file.set_when_modified(tfs_file.when_modified.duration_since(UNIX_EPOCH)?.as_secs());
        capnp_file.set_when_changed(tfs_file.when_changed.duration_since(UNIX_EPOCH)?.as_secs());
        let file_tags = &tfs_file.tags.0;
        let mut capnp_tags = capnp_file.init_tags(file_tags.len().try_into()?);
        for (tag_index, file_tag) in file_tags.iter().enumerate() {
            capnp_tags.set(tag_index.try_into()?, file_tag.get_id());
        } 
    }

    let mut capnp_tags = capnp_filesystem.reborrow().init_tags(tfs_tags.len().try_into()?);
    for (tag_index, tfs_tag) in tfs_tags.iter().enumerate() {
        let mut capnp_tag = capnp_tags.reborrow().get(tag_index.try_into()?);
        capnp_tag.set_name(tfs_tag.name.clone());
        capnp_tag.set_inode(tfs_tag.inode.get_id());
        capnp_tag.set_owner(tfs_tag.owner);
        capnp_tag.set_group(tfs_tag.group);
        capnp_tag.set_permissions(tfs_tag.permissions);
        capnp_tag.set_when_accessed(tfs_tag.when_accessed.duration_since(UNIX_EPOCH)?.as_secs());
        capnp_tag.set_when_modified(tfs_tag.when_modified.duration_since(UNIX_EPOCH)?.as_secs());
        capnp_tag.set_when_changed(tfs_tag.when_changed.duration_since(UNIX_EPOCH)?.as_secs());
    }

    serialize_packed::write_message(write_location, &capnp_message)?;

    Ok(())
}

#[test]
fn running_serialize_tag_filesystem() {
    let mut x = vec![];
    serialize_tag_filesystem(&mut x, vec![
        &TfsFile::builder()
            .name(String::from("Poop"))
            .inode(101.try_into().unwrap())
            .owner(1000)
            .group(1000)
            .build()
    ], vec![], vec![]);
    // TODO: Read up on Cursor::new
    deserialize_tag_filesystem(Cursor::new(x));
}
