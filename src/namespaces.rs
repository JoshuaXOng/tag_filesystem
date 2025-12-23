use std::{collections::HashMap, fmt::Display, time::SystemTime};

use bon::Builder;
use fuser::FileType;

use crate::{entries::TfsEntry, errors::ResultBtAny, inodes::{NamespaceInode, TagInodes},
    wrappers::write_iter};

pub const DEFAULT_NAMESPACE_PERMISSIONS: u16 = 0o777;

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct TfsNamespace {
    pub name: String,
    pub inode: NamespaceInode,
    pub tags: TagInodes,
    pub owner: u32,
    pub group: u32,
    #[builder(default = DEFAULT_NAMESPACE_PERMISSIONS)]
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

impl TfsEntry for TfsNamespace {
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

impl<'a> From<&'a TfsNamespace> for &'a TagInodes {
    fn from(value: &'a TfsNamespace) -> Self {
        &value.tags
    }
}

impl From<TfsNamespace> for TagInodes {
    fn from(value: TfsNamespace) -> Self {
        value.tags
    }
}

impl Display for TfsNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(id={}, tags={})", self.name, self.inode, self.tags)
    }
}

#[derive(Debug)]
pub struct IndexedNamepsaces {
    namespaces: HashMap<NamespaceInode, TfsNamespace>
}

impl IndexedNamepsaces {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new()
        }
    }

    pub fn get_by_inode(&self, namespace_inode: &NamespaceInode) -> ResultBtAny<&TfsNamespace> {
        self.namespaces.get(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode).into())
    }

    pub fn get_by_inode_id(&self, inode_id: u64) -> ResultBtAny<&TfsNamespace> {
        let namespace_inode = NamespaceInode::try_from(inode_id)?;
        self.namespaces.get(&namespace_inode)
            .ok_or(format!("Namespace with inode `{namespace_inode}` does not \
                exist.").into())
    }

    pub fn get_by_inode_mut(&mut self, namespace_inode: &NamespaceInode)
    -> ResultBtAny<NamespaceUpdate<'_>> {
        Ok(self.namespaces
            .get_mut(&namespace_inode)
            .ok_or(Self::get_namespace_404_message(namespace_inode))?
            .into())
    }

    fn get_namespace_404_message(namespace_inode: &NamespaceInode) -> String {
        format!("Namespace id `{namespace_inode}` does not exist.")
    }

    pub fn get_all(&self) -> impl Iterator<Item = &TfsNamespace> {
        self.namespaces.values()
    }

    pub fn get_map(&self) -> &HashMap<NamespaceInode, TfsNamespace> { 
        &self.namespaces 
    }

    pub fn get_free_inode(&self) -> ResultBtAny<NamespaceInode> {
        let inodes_inuse = self.namespaces.keys();
        NamespaceInode::try_from_free_inodes(inodes_inuse)
    }

    pub fn add(&mut self, to_add: TfsNamespace) -> ResultBtAny<&TfsNamespace> {
        let namespace_inode = to_add.inode;

        let does_conflict = self.namespaces.get(&namespace_inode).is_some();
        if does_conflict {
            Err(format!("Namespace with id `{}` already exists.", namespace_inode))?;
        }
        
        self.namespaces.insert(namespace_inode, to_add);
        Ok(self.namespaces.get(&namespace_inode)
            .expect("To have just inserted with inode prior."))
    }

    pub fn do_for_all<'a, T>(&'a mut self,
        mut to_do: impl FnMut(NamespaceUpdate) -> T + 'a)
        -> impl Iterator<Item = T> + 'a
    {
        self.namespaces.values_mut()
            .map(move |namespace| to_do(namespace.into()))
    }
}

impl Display for IndexedNamepsaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_iter(f, ('[', ']'), self.namespaces.values())
    }
}

pub struct NamespaceUpdate<'a> {
    pub name: &'a mut String,
    inode: &'a NamespaceInode,
    pub tags: &'a mut TagInodes
}

impl<'a> NamespaceUpdate<'a> {
    pub fn get_inode(&self) -> &NamespaceInode {
        &self.inode
    }
}

impl<'a> From<&'a mut TfsNamespace> for NamespaceUpdate<'a> {
    fn from(value: &'a mut TfsNamespace) -> Self {
        NamespaceUpdate {
            name: &mut value.name,
            inode: &mut value.inode,
            tags: &mut value.tags
        }
    }
}
