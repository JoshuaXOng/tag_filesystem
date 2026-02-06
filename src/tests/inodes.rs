use crate::{
    errors::ResultBtAny,
    inodes::{FileInode, NamespaceInode, TagInode},
};

#[test]
fn creating_file_inodes() -> ResultBtAny<()> {
    let file_inode = FileInode::try_from(3)?;
    assert_eq!(file_inode.get_id(), 3);

    let file_inode = FileInode::try_from(6)?;
    assert_eq!(file_inode.get_id(), 6);

    let file_inode = FileInode::try_from(9)?;
    assert_eq!(file_inode.get_id(), 9);

    let expectation = "Needs to be remainder 0 after mod 3.";
    FileInode::try_from(4).expect_err(expectation);
    FileInode::try_from(5).expect_err(expectation);

    Ok(())
}

#[test]
fn creating_tag_inodes() -> ResultBtAny<()> {
    let tag_inode = TagInode::try_from(4)?;
    assert_eq!(tag_inode.get_id(), 4);

    let tag_inode = TagInode::try_from(7)?;
    assert_eq!(tag_inode.get_id(), 7);

    let tag_inode = TagInode::try_from(10)?;
    assert_eq!(tag_inode.get_id(), 10);

    let expectation = "Needs to be remainder 1 after mod 3.";
    TagInode::try_from(5).expect_err(expectation);
    TagInode::try_from(6).expect_err(expectation);

    Ok(())
}

#[test]
fn creating_namespace_inodes_() -> ResultBtAny<()> {
    let namespace_inode = NamespaceInode::try_from(5)?;
    assert_eq!(namespace_inode.get_id(), 5);

    let namespace_inode = NamespaceInode::try_from(8)?;
    assert_eq!(namespace_inode.get_id(), 8);

    let namespace_inode = NamespaceInode::try_from(11)?;
    assert_eq!(namespace_inode.get_id(), 11);

    let expectation = "Needs to be remainder 2 after mod 3.";
    NamespaceInode::try_from(6).expect_err(expectation);
    NamespaceInode::try_from(7).expect_err(expectation);

    Ok(())
}
