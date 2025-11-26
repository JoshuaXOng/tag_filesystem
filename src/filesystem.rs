use std::{fmt::Display, path::PathBuf, time::SystemTime};

use bon::bon;
use fuser::FileAttr;
use tracing::{instrument, warn};

#[cfg(test)]
use crate::storage::StubStorage;
use crate::{entries::TfsEntry, errors::Result_, files::{IndexedFiles, TfsFile}, inodes::{FileInode, NamespaceInode, TagInode, TagInodes}, path_::{format_tags, parse_tags}, namespaces::{self, IndexedNamepsaces}, storage::{DelegateStorage, TfsStorage}, tags::{IndexedTags, TfsTag}, wrappers::VecWrapper};

// TODO: Save after each change, how to mmap to disk?
#[derive(Debug)]
pub struct TagFilesystem<Storage: TfsStorage> {
    files: IndexedFiles,
    tags: IndexedTags,
    // TODO: Should store parent to allow multi depth namespaces.
    namespaces: IndexedNamepsaces,
    storage: Storage
}

impl TagFilesystem<DelegateStorage> {
    pub fn try_new(mount_path: &PathBuf) -> Result_<Self> {
        Ok(Self {
            files: IndexedFiles::new(),
            tags: IndexedTags::new(),
            namespaces: IndexedNamepsaces::new(),
            storage: DelegateStorage::try_new(mount_path)?
        })
    }
}

#[bon]
impl<Storage: TfsStorage> TagFilesystem<Storage> {
    pub fn get_files(&self) -> &IndexedFiles {
        &self.files
    }

    pub fn get_free_file_inode(&self) -> Result_<FileInode> {
        self.files.get_free_inode()
    }

    pub fn get_tags(&self) -> &IndexedTags {
        &self.tags
    }

    pub fn get_free_tag_inode(&self) -> Result_<TagInode> {
        self.tags.get_free_inode()
    }

    pub fn get_namespaces(&self) -> &IndexedNamepsaces {
        &self.namespaces
    }

    pub fn get_file_by_name_and_namespace_inode(&self, file_name: &str,
        namespace_inode: &NamespaceInode) -> Result_<&TfsFile>
    {
        let namespace_tags = &self.namespaces.get_by_inode(namespace_inode)?.tags;
        self.files.get_by_name_and_tags(file_name, namespace_tags)
            .ok_or(format!("File with name `{file_name}` and tags `{namespace_tags}` \
                does not exist.").into())
    }

    pub fn get_files_by_namespace_inode(&self, namespace_inode: &NamespaceInode)
    -> Result_<Vec<&TfsFile>> {
        let namespace_tags = &self.namespaces.get_by_inode(namespace_inode)?.tags;
        Ok(self.files.get_by_tags(namespace_tags))
    }

    pub fn get_inrange_tags<'a>(&self, tag_inodes: impl Into<&'a TagInodes>)
    -> Result_<Vec<&TfsTag>> {
        let tag_inodes = tag_inodes.into();

        let mut inrange_tags = vec![];

        for tag_inode in &tag_inodes.0 {
            inrange_tags.push(self.get_tags()
                .get_by_inode(tag_inode)
                .ok_or(format!("Tag inode `{tag_inode}` does not exist."))?);
        }

        inrange_tags.extend(self.get_neighbour_tags(tag_inodes)?);

        Ok(inrange_tags)
    }

    pub fn get_neighbour_tags<'a>(&self, tag_inodes: impl Into<&'a TagInodes>)
    -> Result_<Vec<&TfsTag>> {
        let tag_inodes = tag_inodes.into();

        let neighbour_inodes = self.files.get_neighbour_tag_inodes(tag_inodes);
        neighbour_inodes.0.iter()  
            .map(|inode| self.tags.get_by_inode(inode)
                .ok_or(format!("Tag inode with id `{}` \
                    does not exist.", inode.get_id()).into()))
            .collect()
    }

    // TODO: Convert silent warn to error return.
    #[instrument]
    fn get_namespace_string_from_tags(filesystem_tags: &IndexedTags, tag_inodes: &TagInodes)
    -> String {
        format_tags(tag_inodes.0.iter()
            .filter_map(|inode| 
                filesystem_tags.get_by_inode(&inode)
                    .or_else(|| {
                        warn!("Likely bug, tag ids should always be valid.");
                        None
                    }))
            .map(|tag| tag.name.as_str()))
    }

    pub fn get_fuser_attributes(&self, inode_id: u64) -> Result_<FileAttr> {
        FileInode::try_from(inode_id)
            .and_then(|inode| self.get_file_fuser(&inode))
            .or(TagInode::try_from(inode_id)
                .and_then(|inode| self.get_tag_fuser(&inode)))
                .or(NamespaceInode::try_from(inode_id)
                    .and_then(|inode| self.get_namespace_fuser(&inode)))
            .map_err(|_| format!("`{inode_id}` is not either of a file,
                tag or namespace inode.").into())
    }

    pub fn get_file_fuser(&self, file_inode: &FileInode) -> Result_<FileAttr> {
        let target_file = self.files.get_by_inode(&file_inode)
            .ok_or(format!("File with inode `{file_inode}` does not exist."))?;
        Ok(Self::to_fuser()
            .tfs_entry(target_file)
            .file_size(self.storage.get_file_size(&file_inode)?)
            .call())
    }

    pub fn get_tag_fuser(&self, tag_inode: &TagInode) -> Result_<FileAttr> {
        let target_tag = self.tags.get_by_inode(&tag_inode)
            .ok_or(format!("Tag with inode `{tag_inode}` does not exist."))?;
        Ok(Self::to_fuser()
            .tfs_entry(target_tag)
            .call())
    }

    pub fn get_namespace_fuser(&self, namespace_inode: &NamespaceInode) -> Result_<FileAttr> {
        Ok(namespaces::get_fuse_attributes(&namespace_inode))
    }

    #[builder]
    fn to_fuser(tfs_entry: &dyn TfsEntry, file_size: Option<u64>) -> FileAttr {
        let file_size = file_size.unwrap_or(0);
        // TODO: What to do with `blocks`, `nlink`, etc.
        FileAttr {
            ino: tfs_entry.get_inode_id(),
            size: file_size,
            blocks: 0,
            atime: tfs_entry.get_when_accessed(),
            mtime: tfs_entry.get_when_modified(),
            ctime: tfs_entry.get_when_changed(),
            crtime: SystemTime::UNIX_EPOCH,
            kind: tfs_entry.get_file_kind(),
            perm: tfs_entry.get_permissions(),
            nlink: 0,
            uid: tfs_entry.get_owner(),
            gid: tfs_entry.get_group(),
            rdev: 0,
            blksize: 0,
            flags: 0,
        }
    }

    fn check_tags_exist(&self, to_check: &TagInodes) -> Result_<()> {
        let doesnt_exist: Vec<_> = to_check.0.iter()
            .filter(|inode| self.tags.get_by_inode(inode)
                .is_none())
            .collect();
        if doesnt_exist.len() > 0 {
            Err(format!(
                "These tag inodes don't exist `{}`.",
                VecWrapper(doesnt_exist)))?;
        }
        Ok(())
    }

    fn check_if_file_is_valid(&self, to_check: &TfsFile) -> Result_<()> {
        if let Some(similar_file) = self.files.get_by_name_and_tags(&to_check.name,
            &to_check.tags)
        {
            let are_files_same = to_check.inode == similar_file.inode;
            if !are_files_same {
                return Err(format!("File with name `{}` and tags `{}` already \
                    exists.", to_check.inode, to_check.tags).into());
            }
        }

        for inrange_tag in self.get_inrange_tags(to_check)? {
            let are_names_name = to_check.name == inrange_tag.name;
            if are_names_name {
                return Err(format!("File name is same as one of it's tags \
                    or neighbouring tags, `{}`.",
                    to_check.name).into());
            }
        }
        Ok(())
    }

    fn check_if_tag_is_valid<'a>(&self, tag_inode: impl Into<&'a TagInode>) -> Result_<()> {
        let tag_inode = tag_inode.into();
        let target_tag = self.tags.get_by_inode(tag_inode)
            .ok_or(format!("Tag with inode `{tag_inode}` does not exist."))?;
        self.check_if_tag_is_valid_(target_tag)
    }

    fn check_if_tag_is_valid_(&self, to_check: &TfsTag) -> Result_<()> {
        for tfs_tag in self.tags.get_all() {
            let is_same = to_check.inode == tfs_tag.inode;
            let is_colliding = to_check.name == tfs_tag.name; 
            if !is_same && is_colliding {
                return Err(format!("Tag already exists with name `{}`",
                    to_check.name).into());
            }
        }

        for tag_inodes in self.files.get_tag_sets() {
            if !tag_inodes.0.contains(&to_check.inode) {
                continue;
            }

            for file in self.files.get_by_tags(&tag_inodes) {
                let is_colliding = to_check.name == file.name;
                if is_colliding {
                    return Err(format!("Tag has same name as file w/ this tag, `{}`.",
                        to_check.name).into());
                }
            }

            let mut tag_inodes = tag_inodes.clone();
            tag_inodes.0.remove(&to_check.inode);
            for file in self.files.get_by_tags(&tag_inodes) {
                let is_colliding = to_check.name == file.name;
                if is_colliding {
                    return Err(format!("Tag has same name as neighbouring file, `{}`.",
                        to_check.name).into());
                }
            }
        }

        for untagged_file in self.files.get_by_tags(&TagInodes::new()) {
            let is_colliding = to_check.name == untagged_file.name;
            if is_colliding {
                return Err(format!("Tag has same name as untagged file, `{}`.",
                    to_check.name).into());
            }
        }
        Ok(())
    }

    pub fn get_storage(&self) -> &dyn TfsStorage {
        &self.storage
    }

    pub fn add_file(&mut self, to_add: TfsFile) -> Result_<&TfsFile> {
        self.check_if_file_is_valid(&to_add)?;
        self.write_to_file(&to_add.inode, 0, &[])?;
        self.files.add(to_add)
    }

    pub fn add_tag(&mut self, to_add: TfsTag) -> Result_<&TfsTag> {
        self.check_if_tag_is_valid_(&to_add)?;
        self.tags.add(to_add)
    }

    pub fn write_to_file(&mut self, file_inode: &FileInode, start_position: u64, to_write: &[u8])
    -> Result_<()> {
        self.storage.write(file_inode, start_position, to_write)
    }
    
    pub fn move_file<'a>(&mut self,
        old_tags: impl Into<&'a TagInodes>, old_name: &str,
        new_tags: impl Into<TagInodes>, new_name: String)
    -> Result_<()> {
        let old_tags = old_tags.into();
        let new_tags = new_tags.into();

        self.files.do_by_name_and_tags(old_name, old_tags, |mut file| {
            file.try_set_name(new_name.clone())?;
            file.try_set_tags(new_tags.clone())
        })
            .flatten()?;
        let modified_file = self.files.get_by_name_and_tags(&new_name, &new_tags)
            .expect("To have just set name and tags prior.");

        let e = self.check_if_file_is_valid(modified_file)
            .err();
        let errors = new_tags.0.iter()
            .filter_map(|inode| self.check_if_tag_is_valid(inode).err())
            .collect::<Vec<_>>();
        if e.is_some() || !errors.is_empty() {
            self.files.do_by_name_and_tags(&new_name, &new_tags, |mut file| {
                file.try_set_name(old_name.to_string())?;
                file.try_set_tags(old_tags.clone())
            })
                .flatten()
                .expect("To have nothing take up old name and tags in the \
                    meanwhile.");
            let mut _e = String::from("Name and tag(s) combination is invalid.");
            if let Some(e) = e { _e += &format!(" {e}") }
            for e in errors { _e += &format!(" {e}") }
            return Err(_e.into());
        }

        Ok(())
    }

    pub fn rename_tag(&mut self, old_name: &str, new_name: String) -> Result_<()> {
        let tag_inode = self.tags.get_by_name(old_name)
            .ok_or(format!("Tag `{old_name}` does not exist"))?
            .inode;
        self.tags.do_by_inode(&tag_inode, |mut tag| tag.try_set_name(new_name))
            .flatten()?;
        
        let e = self.check_if_tag_is_valid(&tag_inode);
        if e.is_err() {
            self.tags.do_by_inode(&tag_inode,
                |mut tag| tag.try_set_name(old_name.to_string()))
                .flatten()
                .expect("To have just indexed with the same inode. To be reverting \
                    to an unused name.");
            return e;
        }

        self.namespaces.do_for_all(|namespace_update| {
            if namespace_update.tags.0.contains(&tag_inode) {
                let namespace_string = Self::get_namespace_string_from_tags(
                    &self.tags, namespace_update.tags);
                *namespace_update.name = namespace_string;
            }
        });

        Ok(())
    }

    pub fn insert_namespace(&mut self, namespace_string: String) -> Result_<NamespaceInode> {
        let namespace_tags = parse_tags(&namespace_string);

        let mut _namespace_tags = TagInodes::new(); 
        for namespace_tag in namespace_tags {
            let namespace_tag = self.tags.get_by_name(namespace_tag)
                .ok_or(format!("`{namespace_tag}` does not exist."))?;

            _namespace_tags.0.insert(namespace_tag.inode);
        }

        Ok(self.namespaces.insert_limited(namespace_string, _namespace_tags))
    }

    pub fn insert_namespace_(&mut self, tag_inodes: TagInodes) -> NamespaceInode {
        self.namespaces.insert_limited(
            Self::get_namespace_string_from_tags(&self.tags, &tag_inodes),
            tag_inodes)
    }
    
    pub fn remove_file_by_name_and_tags<'a>(&mut self, file_name: &str,
        tag_inodes: impl Into<&'a TagInodes>)
    -> Result_<TfsFile> {
        let tag_inodes = tag_inodes.into();
        let removed_file = self.files.remove_by_name_and_tags(file_name, tag_inodes)
            .ok_or(format!("No file matching name `{file_name}` and tag inodes \
                `{tag_inodes}`."))?;
        self.storage.delete(&removed_file.inode)?;
        Ok(removed_file)
    }

    #[instrument(skip_all, fields(?tag_name))]
    pub fn delete_tag(&mut self, tag_name: &str) -> Result_<TfsTag> {
        let removed_tag = self.tags.remove_by_name(&tag_name)
            .ok_or(format!("Tag `{tag_name}` does not exist."))?;
        let tag_sets: Vec<_> = self.files.get_tag_sets()
            .into_iter()
            .cloned()
            .collect();
        for tag_set in tag_sets {
            if !tag_set.0.contains(&removed_tag.inode) {
                continue;
            }
            
            self.files.do_by_tags(&tag_set, |target_files| {
                *target_files = target_files.drain()
                    .map(|mut file| {
                        let did_remove = file.tags.0.remove(&removed_tag.inode);
                        if !did_remove {
                            warn!("Expected to have filtered for 
                                files that have the tag.");
                        }
                        file
                    })
                    .collect();
            })?;
        }

        self.namespaces.do_for_all(|namespace_update| {
            if namespace_update.tags.0.remove(&removed_tag.inode) {
                let namespace_string = Self::get_namespace_string_from_tags(
                    &self.tags, namespace_update.tags);
                *namespace_update.name = namespace_string;
            }
        });

        Ok(removed_tag)
    }
}

impl<Storage: TfsStorage> Display for TagFilesystem<Storage> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagFilesystem(")?;
        write!(f, "files={}, ", self.files)?;
        write!(f, "tags={}, ", self.tags)?;
        write!(f, "namespaces={}", self.namespaces)?;
        write!(f, ")")
    }
}

#[cfg(test)]
impl TagFilesystem<StubStorage> {
    pub fn new() -> Self {
        Self {
            files: IndexedFiles::new(),
            tags: IndexedTags::new(),
            namespaces: IndexedNamepsaces::new(),
            storage: StubStorage,
        }
    }
}
