use std::{collections::{HashMap, HashSet}, fmt::Display, time::SystemTime};

use bon::Builder;
use fuser::FileType;

use crate::{entries::TfsEntry, errors::ResultBtAny, inodes::{FileInode, TagInodes}, unwrap_or,
    wrappers::{write_btreeset, write_iter, VecWrapper}};

// TODO: Figure out eval steps. File inheriting perms
// from directory etc., maybe rename - same with Tag builder. 
#[derive(Builder, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone)]
#[builder(on(String, into))]
pub struct TfsFile {
    pub name: String,
    pub inode: FileInode,
    pub owner: u32,
    pub group: u32,
    #[builder(default = 0o640)]
    pub permissions: u16,
    #[builder(default = SystemTime::now())]
    pub when_accessed: SystemTime,
    #[builder(default = SystemTime::now())]
    pub when_modified: SystemTime,
    #[builder(default = SystemTime::now())]
    pub when_changed: SystemTime,
    #[builder(default = TagInodes::new())]
    pub tags: TagInodes,
}

impl TfsEntry for TfsFile {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_inode_id(&self) -> u64 {
        self.inode.get_id()
    }

    fn get_owner(&self) -> u32 {
        self.owner
    }

    fn get_group(&self) -> u32 {
        self.group
    }

    fn get_permissions(&self) -> u16 {
        self.permissions
    }

    fn get_file_kind(&self) -> FileType {
        FileType::RegularFile
    }

    fn get_when_accessed(&self) -> SystemTime {
        self.when_accessed
    }

    fn get_when_modified(&self) -> SystemTime {
        self.when_modified
    }

    fn get_when_changed(&self) -> SystemTime {
        self.when_changed
    }
}

impl<'a> From<&'a TfsFile> for &'a TagInodes {
    fn from(value: &'a TfsFile) -> Self {
        &value.tags
    }
}

impl Display for TfsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(id={}, tags=", self.name, self.inode)?;
        write_btreeset(f, &self.tags.0)?;
        write!(f, ")")
    }
}

type ByInode = HashMap<FileInode, TfsFile>;
type ByTags = HashMap<TagInodes, Vec<FileInode>>;
type ByNameAndTags = HashMap<(String, TagInodes), FileInode>;

#[derive(Debug, Clone)]
pub struct IndexedFiles {
    files: ByInode,
    by_tags: ByTags, 
    by_name_and_tags: ByNameAndTags
}

impl IndexedFiles {
    pub fn new() -> Self {
        Self {
            files: ByInode::new(),
            by_tags: ByTags::new(),
            by_name_and_tags: ByNameAndTags::new(),
        }
    }

    pub fn get_by_inode(&self, file_inode: &FileInode) -> Option<&TfsFile> {
        self.files.get(file_inode)
    }

    pub fn get_by_inode_id(&self, inode_id: u64) -> ResultBtAny<&TfsFile> {
        let file_inode = FileInode::try_from(inode_id)?;
        self.files.get(&file_inode)
            .ok_or(format!("File with inode `{file_inode}` does not exist.").into())
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

    pub fn get_by_name_and_tags(&self, file_name: &str, file_tags: &TagInodes)
    -> Option<&TfsFile> {
        let file_inode = self.by_name_and_tags
            .get(&(file_name.to_string(), file_tags.clone()))?;

        self.files.get(file_inode)
    }

    fn get_by_name_and_tags_mut(&mut self, file_name: &str, file_tags: &TagInodes)
    -> Option<&mut TfsFile> {
        let file_inode = self.by_name_and_tags
            .get(&(file_name.to_string(), file_tags.clone()))?;

        self.files.get_mut(file_inode)
    }

    pub fn get_all(&self) -> Vec<&TfsFile> {
        self.files.values().collect()
    }

    fn get_all_mut(&mut self) -> Vec<&mut TfsFile> {
        self.files.values_mut().collect()
    }

    pub fn get_tag_sets(&self) -> Vec<&TagInodes> {
        self.by_tags.iter()
            .filter(|(_, files)| !files.is_empty())
            .map(|(tags, _)| tags)
            .collect()
    }

    pub fn get_neighbour_tag_inodes(&self, current_tags: &TagInodes) -> TagInodes {
        let mut neighbour_tags = TagInodes::new(); 
        for tag_set in self.get_tag_sets() {
            if !tag_set.0.is_superset(&current_tags.0) {
                continue
            }
            neighbour_tags.0.extend(&tag_set.0 - &current_tags.0);
        }
        neighbour_tags
    }

    pub fn get_inuse_inodes(&self) -> Vec<&FileInode> {
        self.files.keys().collect()
    }

    pub fn get_free_inode(&self) -> ResultBtAny<FileInode> {
        let inodes_inuse = self.get_inuse_inodes();
        FileInode::try_from_free_inodes(inodes_inuse)
    }

    fn will_collide(&self, check_for: &TfsFile) -> ResultBtAny<()> {
        Self::_will_collide(&self.files, &self.by_tags, &self.by_name_and_tags,
            &check_for.name, &check_for.inode, &check_for.tags)
    }

    fn _will_collide(files: &ByInode, by_tags: &ByTags, by_name_and_tags: &ByNameAndTags,
        name: &str, inode: &FileInode, tags: &TagInodes
    ) -> ResultBtAny<()> {
        let does_inode = files.contains_key(&inode);
        let do_tags = by_tags.get(&tags)
            .map(|inodes| inodes.contains(&inode))
            .unwrap_or(false);
        let does_name_and_tags = by_name_and_tags
            .contains_key(&(name.to_string(), tags.clone()));
        if does_inode || do_tags || does_name_and_tags {
            Err(format!("Collisions on inode, tags, name and tags: {}, {}, {}",
                does_inode, do_tags, does_name_and_tags))?;
        }
        Ok(())
    }

    pub fn do_by_inode<T>(&mut self, file_inode: &FileInode, to_do: impl FnOnce(FileUpdate) -> T)
    -> ResultBtAny<T> {
        self.do_or_rollback(file_inode, to_do)
    }

    pub fn do_by_tags<T>(&mut self, file_tags: &TagInodes,
        to_do: impl FnOnce(&mut HashSet<TfsFile>) -> T
    ) -> ResultBtAny<T> {
        let target_inodes = self.by_tags.get(file_tags)
            .ok_or(format!("No files with tags `{file_tags}`."))?
            .clone();
        let to_return = self.do_or_complete_rollback_bulk(&target_inodes.into_iter()
            .collect(), to_do)?;
        Ok(to_return)
    }

    pub fn do_by_name_and_tags<T>(&mut self, file_name: &str, file_tags: &TagInodes,
        to_do: impl FnOnce(FileUpdate) -> T
    ) -> ResultBtAny<T> {
        let target_inode = *self.by_name_and_tags.get(&(
            file_name.to_string(),
            file_tags.clone()))
            .ok_or(format!(
                "No file with name and tags: `{}` and `{}`.",
                file_name, file_tags))?;
        self.do_or_rollback(&target_inode, to_do)
    }

    fn do_or_rollback<T>(&mut self, file_inode: &FileInode, to_do: impl FnOnce(FileUpdate) -> T)
    -> ResultBtAny<T> {
        let mut target_file = self.remove_by_inode(file_inode)
            .ok_or(format!("File with inode `{file_inode}` does not exist."))?;
        let callback_return = to_do(FileUpdate {
            files: &self.files,
            by_tags: &self.by_tags,
            by_name_and_tags: &self.by_name_and_tags,
            name: &mut target_file.name,
            inode: &mut target_file.inode,
            owner: &mut target_file.owner,
            group: &mut target_file.group,
            permissions: &mut target_file.permissions,
            when_accessed: &mut target_file.when_accessed,
            when_modified: &mut target_file.when_modified,
            when_changed: &mut target_file.when_changed,
            tags: &mut target_file.tags
        });
        self.add(target_file)?;
        Ok(callback_return)
    }

    // TODO: For other methods and this, return iter instead of Vec, Hashset?
    fn do_or_partial_rollback_bulk<T>(&mut self, file_inodes: &HashSet<FileInode>,
        mut to_do: impl FnMut(FileUpdate) -> T)
    -> ResultBtAny<HashMap<FileInode, T>> {
        let dont_exist = file_inodes.iter()
            .filter(|inode| self.get_by_inode(inode)
                .is_none())
            .collect::<Vec<_>>();
        if !dont_exist.is_empty() {
            return Err(format!("Some inodes `{}` in the argument do not exist.",
                VecWrapper(dont_exist)).into());
        }

        let mut callback_returns = HashMap::new();
        for file_inode in file_inodes {
            callback_returns.insert(*file_inode,
                self.do_or_rollback(file_inode, &mut to_do)?);
        }

        Ok(callback_returns)
    }

    fn do_or_complete_rollback_bulk<T>(&mut self, file_inodes: &HashSet<FileInode>,
        to_do: impl FnOnce(&mut HashSet<TfsFile>) -> T)
    -> ResultBtAny<T> {
        let dont_exist = file_inodes.iter()
            .filter(|inode| self.get_by_inode(inode)
                .is_none())
            .collect::<Vec<_>>();
        if !dont_exist.is_empty() {
            return Err(format!("Some inodes `{}` in the argument do not exist.",
                VecWrapper(dont_exist)).into());
        }

        let mut for_modification = HashSet::new(); 
        for file_inode in file_inodes {
            for_modification.insert(self.remove_by_inode(file_inode)
                .expect("To have checked all inodes correspond with an \
                    existing file."));
        }
        let original_files = for_modification.clone(); 
        let callback_return = to_do(&mut for_modification);

        let mut is_any_conflicts = false;
        let mut modified_files = IndexedFiles::new();
        for modified_file in for_modification {
            if self.will_collide(&modified_file).is_err() 
            || modified_files.add(modified_file).is_err() {
                is_any_conflicts = true;
            }
        }

        let nonconflicting_files = if is_any_conflicts { original_files }
        else { modified_files.files.into_values().collect() };
        for nonconflicting_file in nonconflicting_files {
            self.add_unchecked(nonconflicting_file);
        }

        Ok(callback_return)
    }

    pub fn add(&mut self, to_add: TfsFile) -> ResultBtAny<&TfsFile> {
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

    pub fn remove_by_name_and_tags(&mut self, file_name: &str, file_tags: &TagInodes)
    -> Option<TfsFile> {
        let to_remove = self.by_name_and_tags
            .get(&(file_name.to_string(), file_tags.clone()))?
            .clone();
        self.remove_by_inode(&to_remove)
    }
}

impl Display for IndexedFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_iter(f, ('[', ']'), self.files.values())
    }
}

pub struct FileUpdate<'a, 'b> {
    files: &'a ByInode,
    by_tags: &'a ByTags, 
    by_name_and_tags: &'a ByNameAndTags, 

    name: &'b mut String,
    inode: &'b mut FileInode,
    pub owner: &'b mut u32,
    pub group: &'b mut u32,
    pub permissions: &'b mut u16,
    pub when_accessed: &'b SystemTime,
    pub when_modified: &'b SystemTime,
    pub when_changed: &'b SystemTime,
    tags: &'b mut TagInodes,
}

impl<'a, 'b> FileUpdate<'a, 'b> {
    pub fn try_set_name(&mut self, name: String) -> ResultBtAny<()> {
        let original = self.name.clone();
        *self.name = name;
        if let Err(e) = self.will_collide() {
            *self.name = original;
            return Err(e);
        }
        Ok(())
    }

    pub fn try_set_inode(&mut self, inode: FileInode) -> ResultBtAny<()> {
        let original = self.inode.clone();
        *self.inode = inode;
        if let Err(e) = self.will_collide() {
            *self.inode = original;
            return Err(e);
        }
        Ok(())
    }

    pub fn try_set_tags(&mut self, tags: TagInodes) -> ResultBtAny<()> {
        let original = self.tags.clone();
        *self.tags = tags;
        if let Err(e) = self.will_collide() {
            *self.tags = original;
            return Err(e);
        }
        Ok(())
    }

    fn will_collide(&self) -> ResultBtAny<()> {
        IndexedFiles::_will_collide(&self.files, &self.by_tags, &self.by_name_and_tags,
            &self.name, &self.inode, &self.tags)
    }
}

// TODO: Create macro to do something like the below.
//impl<'a> FileUpdate<'a> {
    // Want to only private a small subset of a nested struct's public fields?
    // Can't selectively private a subset of a nested struct's fields.
    // So, make it less effort to declare the fields that should be made
    // public.
    //
    // Where `file` would be a field on the struct that is to be projected.
    // ```project!(file, { 
    //     name[RO]: String[str],
    //     inode[RO]: FileInode,
    //     tags[RO]: TagInodes,
    //     something: String[str],
    //     something_2: String[str],
    // })```
    // will generate something like the below
    // ```
    // fn get_name(&self) -> &str { ... }
    // fn get_inode(&self) -> &FileInode { ... }
    // fn get_tags(&self) -> &TagInodes { ... }
    // fn get_something(&self) -> &str { &self.file.something }
    // fn get_something_mut(&mut self) -> &mut str { &mut self.file.something }
    // fn set_something(&mut self, something: String) { self.file.something = something; }
    // fn get_something_2(&self) -> &str { &self.file.something_2 }
    // fn get_something_2_mut(&mut self) -> &mut str { &mut self.file.something_2 }
    // fn set_something_2(&mut self, something_2: String) { self.file.something_2 = something_2; }
    // ```
    //
    // If any functions, just use `delegate` crate instead.
//}
