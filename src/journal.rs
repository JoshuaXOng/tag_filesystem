use std::{fs::read_to_string, path::PathBuf};

use crate::{files::TfsFile, inodes::{FileInode, NamespaceInode, TagInode},
    namespaces::TfsNamespace, path_::get_configuration_directory, tags::TfsTag};

#[derive(Debug)]
pub struct TfsJournal {
    operations: Vec<TfsOperation>
}

impl TfsJournal {
    const JOURNAL_FILE_NAME: &str = "tfs.journal";

    pub fn new() -> Self {
        let to_journal = Self::get_journal_file_path();
        let journal_content = read_to_string(to_journal);

        Self {
            operations: vec![]
        }
    }
    
    fn get_journal_file_path() -> PathBuf {
        get_configuration_directory().join(Self::JOURNAL_FILE_NAME)
    }

    pub fn get_all_operations(&self) -> Vec<&TfsOperation> {
        todo!()
    }

    pub fn insert_operation(&mut self, tfs_operation: TfsOperation) {
        self.operations.push(tfs_operation);
    }

    pub fn flush_to_file(&self) {
        todo!()
    }
}

#[derive(Debug)]
pub enum TfsOperation {
    // TODO: File content won't be included
    UpsertFile(TfsFile),
    UpsertTag(TfsTag),
    UpsertNamespace(TfsNamespace),
    // TODO: Create central struct
    WriteToFile {
        file_inode: FileInode,
        start_position: u64,
        to_write: Vec<u8>
    },
    RemoveFile {
        remove_inode: FileInode
    },
    RemoveTag {
        remove_inode: TagInode
    },
    RemoveNamespace {
        remove_inode: NamespaceInode
    },
}

impl TfsOperation {
    
}
