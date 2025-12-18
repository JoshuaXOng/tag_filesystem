use std::io::Cursor;

use crate::{files::TfsFile, persistence::{deserialize_tag_filesystem,
    serialize_tag_filesystem}, tags::TfsTag};

#[test]
fn running_tag_filesystem_serdeialization() {
    let mut persistence_location = vec![];
    serialize_tag_filesystem(&mut persistence_location, vec![
            &TfsFile::builder()
                .name(String::from("test_file_a"))
                .inode(101.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsFile::builder()
                .name(String::from("test_file_b"))
                .inode(103.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build()
        ], 
        vec![
            &TfsTag::builder()
                .name(String::from("test_tag_a"))
                .inode(102.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsTag::builder()
                .name(String::from("test_tag_b"))
                .inode(104.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build(),
            &TfsTag::builder()
                .name(String::from("test_tag_c"))
                .inode(106.try_into().unwrap())
                .owner(1000)
                .group(1000)
                .build()
        ]);
    let (recovered_files, recovered_tags) =
        deserialize_tag_filesystem(Cursor::new(persistence_location)).unwrap();
    let (rf, rt) = (recovered_files, recovered_tags);

    assert_eq!(rf.len(), 2);
    assert_eq!(rf[0].name, "test_file_a");
    assert_eq!(rf[0].inode.get_id(), 101);
    assert_eq!(rf[1].name, "test_file_b");
    assert_eq!(rf[1].inode.get_id(), 103);

    assert_eq!(rt.len(), 3);
    assert_eq!(rt[0].name, "test_tag_a");
    assert_eq!(rt[0].inode.get_id(), 102);
    assert_eq!(rt[1].name, "test_tag_b");
    assert_eq!(rt[1].inode.get_id(), 104);
    assert_eq!(rt[2].name, "test_tag_c");
    assert_eq!(rt[2].inode.get_id(), 106);
}
