use std::{collections::HashMap, fmt::Display, time::SystemTime};

use bon::{builder, Builder};
use fuser::FileType;

use crate::{entries::TfsEntry, errors::ResultBtAny, inodes::TagInode, wrappers::write_iter};

#[derive(Builder, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[builder(on(String, into))]
pub struct TfsTag {
    pub name: String,
    pub inode: TagInode,
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
    #[builder(default = SystemTime::now())]
    pub when_created: SystemTime
}

impl TfsEntry for TfsTag {
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
        FileType::Directory
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

    fn get_when_created(&self) -> SystemTime {
        self.when_created
    }
}

impl Display for TfsTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn TfsEntry).fmt(f)
    }
}

type ByInode = HashMap<TagInode, TfsTag>;
type ByName = HashMap<String, TagInode>;

#[derive(Debug)]
pub struct IndexedTags {
    tags: ByInode,
    by_name: ByName
}

impl IndexedTags {
    pub fn new() -> Self {
        Self {
            tags: ByInode::new(),
            by_name: ByName::new(),
        }
    }

    pub fn get_by_inode(&self, tag_inode: &TagInode) -> Option<&TfsTag> {
        self.tags.get(tag_inode)
    }

    pub fn get_by_inode_id(&self, inode_id: u64) -> ResultBtAny<&TfsTag> {
        let tag_inode = TagInode::try_from(inode_id)?;
        self.tags.get(&tag_inode)
            .ok_or(format!("Tag with inode `{tag_inode}` does not exist.").into())
    }

    fn get_by_inode_mut(&mut self, tag_inode: &TagInode) -> Option<&mut TfsTag> {
        self.tags.get_mut(tag_inode)
    }

    pub fn get_by_name(&self, tag_name: &str) -> Option<&TfsTag> {
        self.by_name.get(tag_name)
            .and_then(|inode| self.tags.get(inode))
    }
    
    fn get_by_name_mut(&mut self, tag_name: &str) -> Option<&mut TfsTag> {
        self.by_name.get(tag_name)
            .and_then(|inode| self.tags.get_mut(inode))
    }

    pub fn get_all(&self) -> impl Iterator<Item = &TfsTag> {
        self.tags.values()
    }

    fn get_all_mut(&mut self) -> impl Iterator<Item = &mut TfsTag> {
        self.tags.values_mut()
    }

    pub fn get_inuse_inodes(&self) -> impl Iterator<Item = &TagInode> {
        self.tags.keys()
    }

    pub fn get_free_inode(&self) -> ResultBtAny<TagInode> {
        let inodes_inuse = self.get_inuse_inodes();
        TagInode::try_from_free_inodes(inodes_inuse)
    }

    pub fn do_by_inode<T>(&mut self, tag_inode: &TagInode, to_do: impl FnOnce(TagUpdate) -> T)
    -> ResultBtAny<T> {
        self.do_or_rollback(tag_inode, to_do)
    }

    pub fn do_by_name<T>( &mut self, tag_name: &str, to_do: impl FnOnce(TagUpdate) -> T)
    -> ResultBtAny<T> {
        let target_inode = *self.by_name.get(tag_name)
            .ok_or(format!(
                "Tag with name `{tag_name}` does not exist."))?;
        self.do_or_rollback(&target_inode, to_do)
    }

    fn do_or_rollback<T>(&mut self, tag_inode: &TagInode, to_do: impl FnOnce(TagUpdate) -> T)
    -> ResultBtAny<T> {
        let mut target_tag = self.remove_by_inode(tag_inode)
            .ok_or(format!("Tag with inode `{tag_inode}` does not exist."))?;
        let callback_return = to_do(TagUpdate {
            tags: &self.tags,
            by_name: &self.by_name,
            name: &mut target_tag.name,
            inode: &mut target_tag.inode,
            owner: &mut target_tag.owner,
            group: &mut target_tag.group,
            permissions: &mut target_tag.permissions,
            when_accessed: &mut target_tag.when_accessed,
            when_modified: &mut target_tag.when_modified,
            when_changed: &mut target_tag.when_changed,
        });
        self.add(target_tag)?;
        Ok(callback_return)
    }

    fn will_collide(&self, check_for: &TfsTag) -> ResultBtAny<()> {
        Self::_will_collide(&self.tags, &self.by_name, &check_for.inode, &check_for.name)
    }

    fn _will_collide(tags: &ByInode, by_name: &ByName, inode: &TagInode, name: &str)
    -> ResultBtAny<()> {
        let does_inode = tags.contains_key(&inode);
        let does_name = by_name.contains_key(name);
        if does_inode || does_name {
            Err(format!("Collisions on inode and name: {}, {}",
                does_inode, does_name))?;
        }
        Ok(())
    }

    pub fn add(&mut self, to_add: TfsTag) -> ResultBtAny<&TfsTag> {
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

    pub fn remove_by_name(&mut self, tag_name: &str) -> Option<TfsTag> {
        let tag_inode = *self.by_name.get(tag_name)?;
        self.remove_by_inode(&tag_inode)
    }
}

impl Display for IndexedTags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
    -> std::fmt::Result {
        write_iter(f, ('[', ']'), self.tags.values())
    }
}

pub struct TagUpdate<'a, 'b> {
    tags: &'a ByInode,
    by_name: &'a ByName,

    name: &'b mut String,
    inode: &'b mut TagInode,
    pub owner: &'b mut u32,
    pub group: &'b mut u32,
    pub permissions: &'b mut u16,
    pub when_accessed: &'b mut SystemTime,
    pub when_modified: &'b mut SystemTime,
    pub when_changed: &'b mut SystemTime,
}

impl<'a, 'b> TagUpdate<'a, 'b> {
    pub fn try_set_name(&mut self, name: String) -> ResultBtAny<()> {
        let original = self.name.clone();
        *self.name = name;
        if let Err(e) = self.will_collide() {
            *self.name = original;
            return Err(e);
        }
        Ok(())
    }

    pub fn try_set_inode(&mut self, inode: TagInode) -> ResultBtAny<()> {
        let original = self.inode.clone();
        *self.inode = inode;
        if let Err(e) = self.will_collide() {
            *self.inode = original;
            return Err(e);
        }
        Ok(())
    }

    fn will_collide(&self) -> ResultBtAny<()> {
        IndexedTags::_will_collide(&self.tags, &self.by_name, self.inode, self.name)
    }
}
