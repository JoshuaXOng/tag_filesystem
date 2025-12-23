use std::io::Cursor;

use crate::{files::TfsFile, filesystem::new_root_fuser_, persistence::{deserialize_tag_filesystem,
    serialize_tag_filesystem, PersistedTfs}, tags::TfsTag};

#[test]
fn running_tag_filesystem_serdeialization() {
    let mut persistence_location = vec![];
    let root_fuser = new_root_fuser_();
    serialize_tag_filesystem(
        &mut persistence_location,
        &root_fuser,
        vec![
            &TfsFile::builder()
                .name(String::from("test_file_a"))
                .inode(3.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsFile::builder()
                .name(String::from("test_file_b"))
                .inode(6.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build()
        ], 
        vec![
            &TfsTag::builder()
                .name(String::from("test_tag_a"))
                .inode(4.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsTag::builder()
                .name(String::from("test_tag_b"))
                .inode(7.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsTag::builder()
                .name(String::from("test_tag_c"))
                .inode(10.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build()
        ]);
    let PersistedTfs { root: _root_fuser, files: recovered_files, tags: recovered_tags }
        = deserialize_tag_filesystem(Cursor::new(persistence_location)).unwrap();
    assert_eq!(root_fuser, _root_fuser);
    let (rf, rt) = (recovered_files, recovered_tags);

    assert_eq!(rf.len(), 2);
    assert_eq!(rf[0].name, "test_file_a");
    assert_eq!(rf[0].inode.get_id(), 3);
    assert_eq!(rf[1].name, "test_file_b");
    assert_eq!(rf[1].inode.get_id(), 6);

    assert_eq!(rt.len(), 3);
    assert_eq!(rt[0].name, "test_tag_a");
    assert_eq!(rt[0].inode.get_id(), 4);
    assert_eq!(rt[1].name, "test_tag_b");
    assert_eq!(rt[1].inode.get_id(), 7);
    assert_eq!(rt[2].name, "test_tag_c");
    assert_eq!(rt[2].inode.get_id(), 10);
}
