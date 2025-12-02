use crate::{entries::TfsEntry, files::TfsFile, filesystem::TagFilesystem, inodes::{FileInode,
    TagInode, TagInodes}, tags::TfsTag};

#[test]
fn displaying_file_things() {
    let file_inode = FileInode::try_from(103).unwrap();
    assert_eq!(format!("{}", file_inode), "103");

    let mut tfs_file = TfsFile::builder()
        .name("test_file")
        .inode(file_inode)
        .owner(1001)
        .group(1001)
        .build();
    assert_eq!(format!("{}", &tfs_file as &dyn TfsEntry),
        "test_file(id=103)");

    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=103, tags={})");
    tfs_file.tags.0.insert(102.try_into().unwrap());
    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=103, tags={ 102 })");
    tfs_file.tags.0.insert(104.try_into().unwrap());
    assert_eq!(format!("{}", &tfs_file),
        "test_file(id=103, tags={ 102, 104 })");
}

#[test]
fn displaying_tag_filesystem() {
    let mut tag_filesystem = TagFilesystem::new();
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[], tags=[], namespaces=[])");

    tag_filesystem.add_file(TfsFile::builder()
        .name("file_1")
        .inode(FileInode::try_from(101).unwrap())
        .owner(1000)
        .group(1000)
        .build());
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[file_1(id=101, tags={})], tags=[], namespaces=[])");

    tag_filesystem.add_tag(TfsTag::builder()
        .name("tag_1")
        .inode(TagInode::try_from(102).unwrap())
        .owner(1000)
        .group(1000)
        .build());
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[file_1(id=101, tags={})], \
            tags=[tag_1(id=102)], \
            namespaces=[])");

    tag_filesystem.insert_namespace(String::from("{ tag_1 }"));
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[file_1(id=101, tags={})], \
            tags=[tag_1(id=102)], \
            namespaces=[{ tag_1 }(id=2, tags={ 102 })])");

    tag_filesystem.move_file(
        &TagInodes::new(), "file_1",
        TagInode::try_from(102).unwrap(), String::from("file_1"));
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[file_1(id=101, tags={ 102 })], \
            tags=[tag_1(id=102)], \
            namespaces=[{ tag_1 }(id=2, tags={ 102 })])");

    tag_filesystem.rename_tag("tag_1", String::from("tag_juan"));
    assert_eq!(format!("{}", tag_filesystem),
        "TagFilesystem(files=[file_1(id=101, tags={ 102 })], \
            tags=[tag_juan(id=102)], \
            namespaces=[{ tag_juan }(id=2, tags={ 102 })])");
}
