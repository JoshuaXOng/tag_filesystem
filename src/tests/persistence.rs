use std::io::Cursor;

use crate::{files::TfsFile, persistence::{deserialize_tag_filesystem, serialize_tag_filesystem}};

#[test]
fn running_serialize_tag_filesystem() {
    let mut x = vec![];
    serialize_tag_filesystem(&mut x, vec![
        &TfsFile::builder()
            .name(String::from("Poop"))
            .inode(101.try_into().unwrap())
            .owner(1000)
            .group(1000)
            .build()
    ], vec![]);
    // TODO: Read up on Cursor::new
    let (a, b) = deserialize_tag_filesystem(Cursor::new(x)).unwrap();
    println!("{:?}", a);
    println!("{:?}", b);
}
