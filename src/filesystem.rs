use std::{ffi::{OsStr, OsString}, fmt};

use tracing::{instrument, warn};

use crate::{errors::Result_, files::{TfsFile, TfsFileBuilder, TfsFiles}, inodes::{QueryInode, TagInodes}, queries::TfsQueries, tags::{TfsTag, TfsTagBuilder, TfsTags}};

#[derive(Debug)]
pub struct TagFilesystem {
    files: TfsFiles,
    // TODO: Need to delete files or update them when tags get updated/deleted.
    tags: TfsTags,
    queries: TfsQueries
}

impl TagFilesystem {
    pub fn new() -> Self {
        Self {
            files: TfsFiles::new(),
            tags: TfsTags::new(),
            queries: TfsQueries::new()
        }
    }

    pub fn get_files(&self) -> &TfsFiles {
        &self.files
    }

    pub fn get_tags(&self) -> &TfsTags {
        &self.tags
    }

    pub fn get_queries(&self) -> &TfsQueries {
        &self.queries
    }

    // TODO: Maybe make TryInto
    pub fn get_file_by_name_and_query_inode(
        &self, file_name: &OsStr, query_inode: &QueryInode)
    -> Result_<Option<&TfsFile>> {
        let tfs_query = self.convert_query_inodes(query_inode)?;
        Ok(self.files.get_by_name_and_tags(file_name, tfs_query))
    }

    pub fn get_files_by_query_inode(&self, query_inode: &QueryInode)
    -> Result_<Vec<&TfsFile>> {
        let tfs_query = self.convert_query_inodes(query_inode)?;
        Ok(self.files.get_by_tags(tfs_query))
    }

    fn convert_query_inodes(&self, query_inode: &QueryInode) 
    -> Result_<&TagInodes> {
        self.queries.as_map()
            .get(&query_inode)
            .ok_or(format!("Query id `{query_inode:?}` does \
                not exist.").into())
    }

    fn check_tags_exist(&self, to_check: &TagInodes) -> Result_<()> {
        let does_exists: Vec<_> = to_check.iter()
            .map(|inode| (
                inode,
                self.tags.get_by_inode(inode).is_some()
            ))
            .collect();
        if does_exists.iter().any(|(_, does)| !does) {
            Err(format!(
                "All tags inodes need to exist: `{:?}`.", does_exists
            ))?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn create_file(
        &mut self, 
        file_name: impl Into<OsString> + fmt::Debug,
    ) -> Result_<&TfsFile> {
        self.create_file_with_tags(file_name, TagInodes::new())
    }

    #[instrument(skip(self))]
    pub fn create_file_with_tags(
        &mut self, 
        file_name: impl Into<OsString> + fmt::Debug,
        file_tags: TagInodes
    ) -> Result_<&TfsFile> {
        let available_inode = self.files.get_free_inode()?;
        self.check_tags_exist(&file_tags)?;

        let new_file = TfsFileBuilder::new(&file_name.into(), &available_inode)
            .set_tags(file_tags)
            .build();
        self.files.add(new_file)
    }

    #[instrument(skip(self))]
    pub fn create_tag(&mut self, tag_name: impl Into<OsString> + fmt::Debug)
    -> Result_<&TfsTag> {
        let available_inode = self.tags.get_free_inode()?;
        let new_tag = TfsTagBuilder::new(&tag_name.into(), &available_inode)
            .build();
        self.tags.add(new_tag)
    }

    pub fn set_file_tags(
        &mut self, file_name: &OsStr,
        file_query: &TagInodes,
        new_tags: TagInodes
    )
    -> Result_<()> {
        self.check_tags_exist(&new_tags)?;

        self.files.do_by_name_and_tags(file_name, &file_query, |to_modify| {
            to_modify.tags = new_tags;
        })?;
        Ok(())
    }

    pub fn insert_query(&mut self, tfs_query: &OsStr) -> Result_<QueryInode> {
        let x = tfs_query.to_string_lossy();
        let query_tags = x
            .trim_matches(|character| match character {
                '{' | '}' | ' ' => true,
                _ => false
            })
            .split(',')
            .map(str::trim)
            .filter(|tag| *tag != "");

        let mut _query_tags = TagInodes::new(); 
        for query_tag in query_tags {
            let query_tag = self.tags.get_by_name(OsStr::new(query_tag))
                .ok_or(format!("`{query_tag}` does not exist."))?;

            _query_tags.insert(query_tag.get_inode());
        }

        Ok(self.queries.insert_limited(_query_tags))
    }
    
    pub fn remove_file_by_name_and_query_inode(
        &mut self, file_name: &OsStr, query_inode: &QueryInode)
    -> Result_<TfsFile> {
        self.files.remove_by_name_and_tags(
            file_name,
            self.queries.as_map()
                .get(query_inode)
                .ok_or(format!("Query of id `{query_inode:?}` does not exist."))?)
            .ok_or(format!("No file matching name `{file_name:?}` and \
                query id `{query_inode:?}`.").into())
    }

    pub fn remove_file_by_name_and_tags(
        &mut self, file_name: &OsStr, file_tags: &TagInodes)
    -> Option<TfsFile> {
        self.files.remove_by_name_and_tags(file_name, file_tags)
    }

    pub fn delete_tag(&mut self, tag_name: impl Into<OsString>)
    -> Result_<TfsTag> {
        let tag_name = tag_name.into();

        let removed_tag = self.tags.remove_by_name(&tag_name)
            .ok_or(format!("Tag `{tag_name:?}` does not exist."))?;
        // TODO: Figure out shorthand for copying vec.
        let tag_sets: Vec<_> = self.files.get_tag_sets()
            .into_iter()
            .map(|tags| tags.clone())
            .collect();
        for tag_set in tag_sets {
            if !tag_set.contains(&removed_tag.get_inode()) {
                continue;
            }
            
            self.files.do_by_tags(&tag_set, |target_file| {
                let did_remove = target_file.tags.remove(&removed_tag.get_inode());
                if !did_remove {
                    warn!("Expected to have filtered for files that have the tag.");
                }
            })?;
        }

        Ok(removed_tag)
    }
}
