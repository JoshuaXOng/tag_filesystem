use std::{collections::HashMap, ffi::{OsStr, OsString}, time::SystemTime};

use fuser::{FileAttr, FileType};

use crate::{entries::TfsEntry, errors::Result_, inodes::{try_from_free_inode, TagInode}};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TfsTag {
    name: OsString,
    inode: TagInode,
    permissions: u16
}

impl TfsTag {
    pub fn get_name(&self) -> &OsStr {
        &self.name
    }

    pub fn get_inode(&self) -> TagInode {
        self.inode
    }

    pub fn get_raw_inode(&self) -> u64 {
        self.inode.get_id()
    }

    pub fn get_attributes(&self) -> FileAttr {
        FileAttr {
            ino: self.inode.get_id(),
            size: 0,
            blocks: 0,
            atime: SystemTime::UNIX_EPOCH,
            mtime: SystemTime::UNIX_EPOCH,
            ctime: SystemTime::UNIX_EPOCH,
            crtime: SystemTime::UNIX_EPOCH,
            kind: FileType::Directory,
            perm: self.permissions,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }
}

impl TfsEntry for TfsTag {
    fn get_name(&self) -> OsString {
        self.get_name().into()
    }

    fn get_raw_inode(&self) -> u64 {
        self.get_inode().get_id()
    }

    fn get_attributes(&self) -> FileAttr {
        self.get_attributes()
    }
}

pub struct TfsTagBuilder {
    name: OsString,
    inode: TagInode,
    permissions: Option<u16>,
}

// TODO: Should delay eval, maybe rename.
impl TfsTagBuilder {
    pub fn new(tag_name: &OsStr, inode_id: &TagInode) -> Self {
        Self {
            name: tag_name.to_owned(),
            inode: inode_id.clone(),
            permissions: None,
        }
    }

    pub fn build(self) -> TfsTag {
        TfsTag {
            name: self.name,
            inode: self.inode,
            permissions: self.permissions.unwrap_or(0o644),
        }
    }
}

#[derive(Debug)]
pub struct TfsTags {
    tags: HashMap<TagInode, TfsTag>,
    by_name: HashMap<OsString, TagInode>, 
}

impl TfsTags {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn get_by_inode(&self, tag_inode: &TagInode) -> Option<&TfsTag> {
        self.tags.get(tag_inode)
    }

    fn get_by_inode_mut(&mut self, tag_inode: &TagInode) -> Option<&mut TfsTag> {
        self.tags.get_mut(tag_inode)
    }

    pub fn get_by_name(&self, tag_name: &OsStr) -> Option<&TfsTag> {
        self.by_name.get(tag_name)
            .and_then(|inode| self.tags.get(inode))
    }
    
    fn get_by_name_mut(&mut self, tag_name: &OsStr) -> Option<&mut TfsTag> {
        self.by_name.get(tag_name)
            .and_then(|inode| self.tags.get_mut(inode))
    }

    pub fn get_all(&self) -> Vec<&TfsTag> {
        self.tags.values().collect()
    }

    fn get_all_mut(&mut self) -> Vec<&mut TfsTag> {
        self.tags.values_mut().collect()
    }

    pub fn get_inuse_inodes(&self) -> Vec<&TagInode> {
        self.tags.keys().collect()
    }

    pub fn get_free_inode(&self) -> Result_<TagInode> {
        let inodes_inuse = self.get_inuse_inodes();
        try_from_free_inode(inodes_inuse)
    }

    pub fn do_by_inode<T>(
        &mut self, tag_inode: &TagInode,
        to_do: impl FnOnce(&mut TfsTag) -> T
    ) -> Result_<T> {
        self.do_or_rollback(tag_inode, to_do)
    }

    pub fn do_by_name<T>(
        &mut self, tag_name: &OsStr,
        to_do: impl FnOnce(&mut TfsTag) -> T
    ) -> Result_<T> {
        let target_inode = self.by_name.get(tag_name)
            .ok_or(format!(
                "Tag with name `{}` does not exist.",
                tag_name.to_string_lossy()
            ))?
            .clone();
        self.do_or_rollback(&target_inode, to_do)
    }

    fn do_or_rollback<T>(
        &mut self, tag_inode: &TagInode,
        to_do: impl FnOnce(&mut TfsTag) -> T
    ) -> Result_<T> {
        let mut target_tag = self.get_by_inode_mut(tag_inode)
            .ok_or(format!("No tag with inode `{tag_inode:?}`."))?;

        let original_values = (target_tag.inode, target_tag.name.clone());
        let to_return = to_do(&mut target_tag);
        let new_values = (target_tag.inode, target_tag.name.clone());

        if original_values != new_values {
            let mut target_tag = self.tags.remove(&original_values.0)
                .expect("To not yet have modified inode key.");
            _ = self.by_name.remove(&original_values.1);

            if let Err(e) = self.will_collide(&target_tag) {
                target_tag.inode = original_values.0;
                target_tag.name = original_values.1;
                self.add_unchecked(target_tag);
                return Err(e);
            }
            self.add_unchecked(target_tag);
        }

        Ok(to_return)
    }

    fn will_collide(&self, check_for: &TfsTag) -> Result_<()> {
        let does_inode = self.tags.contains_key(&check_for.inode);
        let does_name = self.by_name.contains_key(&check_for.name);
        if does_inode || does_name {
            Err(format!(
                "Collisions on inode and name : {:?}",
                (does_inode, does_name)
            ))?;
        }
        Ok(())
    }

    pub fn add(&mut self, to_add: TfsTag) -> Result_<&TfsTag> {
        self.will_collide(&to_add)?;
        Ok(self.add_unchecked(to_add))
    }
    
    fn add_unchecked(&mut self, to_add: TfsTag) -> &TfsTag {
        let inode = to_add.inode;
        let name = to_add.name.clone();

        _ = self.tags.insert(inode, to_add);
        _ = self.by_name.insert(name, inode);

        self.tags.get(&inode)
            .expect("To have just inserted with inode prior.")
    }

    pub fn remove_by_inode(&mut self, tag_inode: &TagInode) -> Option<TfsTag> {
        let to_remove = self.tags.remove(tag_inode)?;
        _ = self.by_name.remove(&to_remove.name);
        Some(to_remove)
    }

    pub fn remove_by_name(&mut self, tag_name: &OsStr) -> Option<TfsTag> {
        let tag_inode = self.by_name.get(tag_name)?;
        self.remove_by_inode(&tag_inode.clone())
    }
}
