use crate::inodes::{FileInode, NamespaceInode, TagInode};

#[test]
fn creating_file_inodes() {
    let file_inode = FileInode::try_from(3).unwrap();
    assert_eq!(file_inode.get_id(), 3);

    let file_inode = FileInode::try_from(6).unwrap();
    assert_eq!(file_inode.get_id(), 6);

    let file_inode = FileInode::try_from(9).unwrap();
    assert_eq!(file_inode.get_id(), 9);

    let expectation = "Needs to be remainder 0 after mod 3.";
    FileInode::try_from(4)
        .expect_err(expectation);
    FileInode::try_from(5)
        .expect_err(expectation);
}

#[test]
fn creating_tag_inodes() {
    let tag_inode = TagInode::try_from(4).unwrap();
    assert_eq!(tag_inode.get_id(), 4);

    let tag_inode = TagInode::try_from(7).unwrap();
    assert_eq!(tag_inode.get_id(), 7);

    let tag_inode = TagInode::try_from(10).unwrap();
    assert_eq!(tag_inode.get_id(), 10);

    let expectation = "Needs to be remainder 1 after mod 3.";
    TagInode::try_from(5)
        .expect_err(expectation);
    TagInode::try_from(6)
        .expect_err(expectation);
}

#[test]
fn creating_namespace_inodes_() {
    let namespace_inode = NamespaceInode::try_from(5).unwrap();
    assert_eq!(namespace_inode.get_id(), 5);

    let namespace_inode = NamespaceInode::try_from(8).unwrap();
    assert_eq!(namespace_inode.get_id(), 8);

    let namespace_inode = NamespaceInode::try_from(11).unwrap();
    assert_eq!(namespace_inode.get_id(), 11);

    let expectation = "Needs to be remainder 2 after mod 3.";
    NamespaceInode::try_from(6)
        .expect_err(expectation);
    NamespaceInode::try_from(7)
        .expect_err(expectation);
}
