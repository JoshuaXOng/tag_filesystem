use crate::{entries::TfsEntry, errors::ResultBtAny, files::TfsFile, filesystem::TagFilesystem, inodes::{FileInode,
    TagInode, TagInodes}, tags::TfsTag};

#[test]
fn displaying_file_things() -> ResultBtAny<()> {
    let file_inode = FileInode::try_from(6)?;
    assert_eq!(format!("{}", file_inode), "6");

    let mut tfs_file = TfsFile::builder()
        .name("test_file")
        .inode(file_inode)
        .owner(1001)
        .group(1001)
        .build();
    assert_eq!(format!("{}", &tfs_file as &dyn TfsEntry),
        "test_file(id=6)");

    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=6, tags={})");
    tfs_file.tags.0.insert(4.try_into()?);
    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=6, tags={ 4 })");
    tfs_file.tags.0.insert(7.try_into()?);
    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=6, tags={ 4, 7 })");

    Ok(())
}

#[test]
fn displaying_tag_filesystem() -> ResultBtAny<()> {
    let mut tag_filesystem = TagFilesystem::new();
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[], tags=[], namespaces=[])");

    let file_1_inode = tag_filesystem.add_file()
        .file_name("file_1")
        .owner_id(1000)
        .group_id(1000)
        .call()?
        .inode
        .get_id();
    assert_eq!(format!("{}", tag_filesystem),
        format!("TagFilesystem(\
            files=[file_1(id={file_1_inode}, tags={{}})], \
            tags=[], \
            namespaces=[])"));

    let tag_1_inode = tag_filesystem.add_tag()
        .tag_name("tag_1")
        .owner_id(1000)
        .group_id(1000)
        .call()?
        .inode
        .get_id();
    assert_eq!(format!("{}", tag_filesystem),
        format!("TagFilesystem(files=[file_1(id={file_1_inode}, tags={{}})], \
            tags=[tag_1(id={tag_1_inode})], \
            namespaces=[])"));

    let namespace_id = tag_filesystem.add_namespace_with_name()
        .namespace_string(String::from("{ tag_1 }"))
        .owner_id(1000)
        .group_id(1000)
        .call()?
        .inode
        .get_id();
    assert_eq!(format!("{}", tag_filesystem),
        format!("TagFilesystem(\
            files=[file_1(id={file_1_inode}, tags={{}})], \
            tags=[tag_1(id={tag_1_inode})], \
            namespaces=[{{ tag_1 }}(id={namespace_id}, tags={{ {tag_1_inode} }})])"));

    tag_filesystem.move_file(
        &TagInodes::new(), "file_1",
        TagInode::try_from(tag_1_inode)?, String::from("file_1"));
    assert_eq!(format!("{}", tag_filesystem),
        format!("TagFilesystem(\
            files=[file_1(id={file_1_inode}, tags={{ {tag_1_inode} }})], \
            tags=[tag_1(id={tag_1_inode})], \
            namespaces=[{{ tag_1 }}(id={namespace_id}, tags={{ {tag_1_inode} }})])"));

    tag_filesystem.rename_tag("tag_1", String::from("tag_juan"));
    assert_eq!(format!("{}", tag_filesystem),
        format!("TagFilesystem(\
            files=[file_1(id={file_1_inode}, tags={{ {tag_1_inode} }})], \
            tags=[tag_juan(id={tag_1_inode})], \
            namespaces=[{{ tag_juan }}(id={namespace_id}, tags={{ {tag_1_inode} }})])"));
    
    Ok(())
}
