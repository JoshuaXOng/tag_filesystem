use std::{collections::{BTreeSet, HashMap, HashSet}, ffi::{OsStr, OsString}, time::SystemTime};

use fuser::{FileAttr, FileType};
use tracing::warn;

use crate::{entries::TfsEntry, errors::Result_, inodes::{try_from_free_inode, FileInode, TagInode, TagInodes}, unwrap_or};

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone)]
pub struct TfsFile {
    pub name: OsString,
    pub inode: FileInode,
    // TODO: Remove, should just be a handle to some other struct that contains.
    content: Vec<u8>,
    blocks: u64,
    time: SystemTime,
    permissions: u16,
    pub tags: TagInodes
}

impl TfsFile {
    pub fn get_name(&self) -> &OsStr {
        &self.name
    }

    pub fn get_inode(&self) -> FileInode {
        self.inode
    }

    pub fn get_tags(&self) -> &TagInodes {
        &self.tags
    }

    pub fn get_attributes(&self) -> FileAttr {
        FileAttr {
            ino: self.inode.get_id(),
            // TODO: This is probably wrong.
            size: (self.content.len() * size_of::<u8>()) as u64,
            blocks: self.blocks,
            atime: SystemTime::UNIX_EPOCH,
            mtime: SystemTime::UNIX_EPOCH,
            ctime: SystemTime::UNIX_EPOCH,
            crtime: SystemTime::UNIX_EPOCH,
            kind: FileType::RegularFile,
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

impl TfsEntry for TfsFile {
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

pub struct TfsFileBuilder {
    name: OsString,
    inode: FileInode,
    content: Vec<u8>,
    blocks: u64,
    time: Option<SystemTime>,
    permissions: Option<u16>,
    tags: TagInodes
}

// TODO: Figure out eval steps. File inheriting perms from dir etc.
impl TfsFileBuilder {
    pub fn new(file_name: &OsStr, inode_id: &FileInode) -> Self {
        Self {
            name: file_name.to_owned(),
            inode: inode_id.clone(),
            content: Vec::new(),
            blocks: 0,
            time: None,
            permissions: None,
            tags: TagInodes::new()
        }
    }

    pub fn build(self) -> TfsFile {
        TfsFile {
            name: self.name,
            inode: self.inode,
            content: self.content,
            blocks: self.blocks,
            time: self.time.unwrap_or(SystemTime::UNIX_EPOCH),
            permissions: self.permissions.unwrap_or(0o664),
            tags: self.tags
        }
    }

    pub fn set_tags(mut self, tags: TagInodes) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Debug)]
pub struct TfsFiles {
    files: HashMap<FileInode, TfsFile>,
    by_tags: HashMap<TagInodes, Vec<FileInode>>, 
    by_name_and_tags: HashMap<(OsString, TagInodes), FileInode>, 
}

// TODO: How to de-dup mut and non-mut code.
//
// Don't return mut references of files.
//
// Go through indexes first, rather than file properties.
impl TfsFiles {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            by_tags: HashMap::new(),
            by_name_and_tags: HashMap::new(),
        }
    }

    pub fn get_by_inode(&self, file_inode: &FileInode) -> Option<&TfsFile> {
        self.files.get(file_inode)
    }

    fn get_by_inode_mut(&mut self, file_inode: &FileInode) -> Option<&mut TfsFile> {
        self.files.get_mut(file_inode)
    }

    pub fn get_by_tags(&self, file_tags: &TagInodes) -> Vec<&TfsFile> {
        let matching_inodes = unwrap_or!(self.by_tags.get(file_tags), {
            return Vec::new();
        });

        matching_inodes.iter()
            .filter_map(|inode| self.files.get(inode))
            .collect()
    }

    fn get_by_tags_mut(&mut self, file_tags: &TagInodes) -> Vec<&mut TfsFile> {
        let matching_inodes = unwrap_or!(self.by_tags.get(file_tags), {
            return Vec::new();
        });

        let mut matching_files = Vec::new();
        for (inode, file) in &mut self.files {
            if matching_inodes.contains(inode) {
                matching_files.push(file)
            }
        }
        matching_files
    }

    pub fn get_by_name_and_tags(&self, file_name: &OsStr, file_tags: &TagInodes)
    -> Option<&TfsFile> {
        let file_inode = self.by_name_and_tags
            .get(&(file_name.into(), file_tags.clone()))?;

        self.files.get(file_inode)
    }

    fn get_by_name_and_tags_mut(&mut self, file_name: &OsStr, file_tags: &TagInodes)
    -> Option<&mut TfsFile> {
        let file_inode = self.by_name_and_tags
            .get(&(file_name.into(), file_tags.clone()));
        let file_inode = match file_inode {
            Some(x) => x,
            None => return None,
        };

        self.files.get_mut(file_inode)
    }

    pub fn get_all(&self) -> Vec<&TfsFile> {
        self.files.values().collect()
    }

    fn get_all_mut(&mut self) -> Vec<&mut TfsFile> {
        self.files.values_mut().collect()
    }

    pub fn get_tag_sets(&self) -> Vec<&TagInodes> {
        self.by_tags.keys()
            .collect()
    }

    pub fn get_inuse_inodes(&self) -> Vec<&FileInode> {
        self.files.keys().collect()
    }

    // TODO: Can optimize finding free inode, binary search type beat.
    pub fn get_free_inode(&self) -> Result_<FileInode> {
        let inodes_inuse = self.get_inuse_inodes();
        try_from_free_inode(inodes_inuse)
    }

    fn will_collide(&self, check_for: &TfsFile) -> Result_<()> {
        let does_inode = self.files.contains_key(&check_for.inode);
        let do_tags = self.by_tags.get(&check_for.tags)
            .map(|inodes| inodes.contains(&check_for.inode))
            .unwrap_or(false);
        let does_name_and_tags = self.by_name_and_tags
            .contains_key(&(check_for.name.clone(), check_for.tags.clone()));
        if does_inode || do_tags || does_name_and_tags {
            Err(format!(
                "Collisions on inode, tags, name and tags: {:?}",
                (does_inode, do_tags, does_name_and_tags)
            ))?;
        }
        Ok(())
    }

    pub fn do_by_inode<T>(
        &mut self, file_inode: &FileInode,
        to_do: impl FnOnce(&mut TfsFile) -> T
    ) -> Result_<T> {
        self.do_or_rollback(file_inode, to_do)
    }

    // TODO: See if can change to a func accepting Vec of muts.
    // TODO: Change to FnOnce.
    // TODO: Rollback to the start.
    pub fn do_by_tags<T>(
        &mut self, file_tags: &TagInodes,
        mut to_do: impl FnMut(&mut TfsFile) -> T
    ) -> Result_<Vec<T>> {
        let target_inodes = self.by_tags.get(file_tags)
            .ok_or(format!("No files with tags `{file_tags:?}`."))?
            .clone();
        let to_return = self.do_or_rollback_bulk(&target_inodes.into_iter()
            .collect(), &mut to_do)?;

        Ok(to_return.into_values()
            .collect())
    }

    pub fn do_by_name_and_tags<T>(
        &mut self, file_name: &OsStr, file_tags: &TagInodes,
        to_do: impl FnOnce(&mut TfsFile) -> T
    ) -> Result_<T> {
        let target_inode = self.by_name_and_tags.get(&(file_name.into(), file_tags.clone()))
            .ok_or(format!(
                "No file with name and tags: `{}` and `{:?}`.",
                file_name.to_string_lossy(), file_tags
            ))?
            .clone();
        self.do_or_rollback(&target_inode, to_do)
    }

    fn do_or_rollback<T>(
        &mut self, file_inode: &FileInode,
        to_do: impl FnOnce(&mut TfsFile) -> T
    ) -> Result_<T> {
        let mut target_file = self.get_by_inode_mut(file_inode)
            .ok_or(format!("No file with inode `{file_inode:?}`."))?;

        let original_values = (target_file.inode, target_file.name.clone(), target_file.tags.clone());
        let to_return = to_do(&mut target_file);
        let new_values = (target_file.inode, target_file.name.clone(), target_file.tags.clone());

        if original_values != new_values {
            let mut target_file = self.files.remove(&original_values.0)
                .expect("To not yet have modified inode key.");
            _ = self.by_tags.remove(&original_values.2);
            _ = self.by_name_and_tags.remove(&(
                original_values.1.clone(), original_values.2.clone()
            ));

            if let Err(e) = self.will_collide(&target_file) {
                target_file.inode = original_values.0;
                target_file.name = original_values.1;
                target_file.tags = original_values.2;
                self.add_unchecked(target_file);
                return Err(e);
            }
            self.add_unchecked(target_file);
        }

        Ok(to_return)
    }

    // TODO: See if you can supply vec as arg.
    // TODO: For other methods, return iter instead of Vec, Hashset?
    fn do_or_rollback_bulk<T>(
        &mut self, file_inodes: &HashSet<FileInode>,
        mut to_do: impl FnMut(&mut TfsFile) -> T
    ) -> Result_<HashMap<FileInode, T>> {
        let (target_inodes, nonexistent_inodes): (HashSet<_>, HashSet<_>) = file_inodes.iter()
            .partition(|inode| self.files.get(inode).is_some());
        if nonexistent_inodes.len() > 0 {
            return Err(format!(
                "Inodes `{nonexistent_inodes:?}` don't exist.").into());
        }
        let expectation = "To be using partition of inodes which exist.";

        let original_files: HashSet<_> = target_inodes.iter()
            .map(|inode| self.files.get(&inode)
                .expect(expectation)
                .clone())
            .collect();

        let mut to_return = HashMap::new();

        let mut modified_files = HashSet::new();
        for target_inode in &target_inodes {
            let mut target_file = self.files.get_mut(&target_inode)
                .expect(expectation);
            to_return.insert(*target_inode, to_do(&mut target_file));
            let target_file = self.files.remove(&target_inode)
                .expect(expectation);
            modified_files.insert(target_file);
        }

        let mut inserted_sofar = HashSet::new();
        for modified_file in modified_files {
            let modified_file = unwrap_or!(self.add(modified_file), e, {
                if inserted_sofar.iter()
                    .map(|inode| self.files.remove(inode))
                    .any(|file| file.is_none())
                {
                    warn!("Bug, the inode should have been inserted prior.");
                };
                
                for original_file in original_files.into_iter() {
                    self.add(original_file)
                        .expect("`add` to have no bugs, and \
                            removed inodes of original files that are \
                            being re-instated.");
                }

                return Err(e);
            });

            let did_insert = inserted_sofar.insert(modified_file.get_inode());
            if !did_insert {
                warn!("Bug, the inode should have been removed prior.");
            }
        }

        Ok(to_return)
    }

    pub fn add(&mut self, to_add: TfsFile) -> Result_<&TfsFile> {
        self.will_collide(&to_add)?;
        Ok(self.add_unchecked(to_add))
    }

    fn add_unchecked(&mut self, to_add: TfsFile) -> &TfsFile {
        let inode = to_add.inode;
        let name = to_add.name.clone();
        let tags = to_add.tags.clone();

        _ = self.files.insert(inode, to_add);
        _ = self.by_name_and_tags.insert((name, tags.clone()), inode);
        self.by_tags.entry(tags)
            .or_insert(vec![])
            .push(inode);

        self.files.get(&inode).expect("To have just inserted with inode prior.")
    }

    pub fn remove_by_inode(&mut self, file_inode: &FileInode) -> Option<TfsFile> {
        let to_remove = self.files.remove(file_inode)?;

        if let Some(inodes) = self.by_tags.get_mut(&to_remove.tags) {
            inodes.retain(|inode| inode != file_inode);
        }

        _ = self.by_name_and_tags
            .remove(&(to_remove.name.clone(), to_remove.tags.clone())); 

        Some(to_remove)
    }

    pub fn remove_by_tags(&mut self, file_tags: &TagInodes) -> Vec<TfsFile> {
        let to_remove = unwrap_or!(self.by_tags.get(file_tags), {
            return Vec::new();
        });
        let to_remove = to_remove.clone();

        to_remove.into_iter()
            .filter_map(|inode| self.remove_by_inode(&inode))
            .collect()
    }

    pub fn remove_by_name_and_tags(
        &mut self, file_name: &OsStr, file_tags: &TagInodes
    ) -> Option<TfsFile> {
        let to_remove = self.by_name_and_tags
            .get(&(file_name.into(), file_tags.clone()))?
            .clone();
        self.remove_by_inode(&to_remove)
    }
}
