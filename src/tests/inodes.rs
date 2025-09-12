use crate::inodes::{FileInode, NamespaceInode, TagInode};

#[test]
fn creating_file_inodes() {
    let file_inode = FileInode::try_from(101).unwrap();
    assert_eq!(file_inode.get_id(), 101);

    let file_inode = FileInode::try_from(103).unwrap();
    assert_eq!(file_inode.get_id(), 103);

    let file_inode = FileInode::try_from(105).unwrap();
    assert_eq!(file_inode.get_id(), 105);

    FileInode::try_from(106)
        .expect_err("File inodes are odd integers.");

    FileInode::try_from(99)
        .expect_err("In namespace region.");
}

#[test]
fn creating_tag_inodes() {
    let tag_inode = TagInode::try_from(102).unwrap();
    assert_eq!(tag_inode.get_id(), 102);

    let tag_inode = TagInode::try_from(104).unwrap();
    assert_eq!(tag_inode.get_id(), 104);

    let tag_inode = TagInode::try_from(106).unwrap();
    assert_eq!(tag_inode.get_id(), 106);

    TagInode::try_from(107)
        .expect_err("Tag inodes are even integers.");

    TagInode::try_from(100)
        .expect_err("In namespace region.");
}

#[test]
fn creating_namespace_inodes() {
    let mut namespace_inode = NamespaceInode::try_from(2).unwrap();
    assert_eq!(namespace_inode.get_id(), 2);
    namespace_inode = namespace_inode.get_next();
    assert_eq!(namespace_inode.get_id(), 3);

    namespace_inode = NamespaceInode::try_from(100).unwrap();
    assert_eq!(namespace_inode.get_id(), 100);
    namespace_inode = namespace_inode.get_next();
    assert_eq!(namespace_inode.get_id(), 2);
}
