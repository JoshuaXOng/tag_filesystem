use std::path::PathBuf;

use crate::path::PathBufExt;

#[test]
fn modifying_path_string() {
    let mut path = PathBuf::new()
        .join("/tmp")
        .join("tfs");

    path = path.join("{}");
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{}");

    path.add_tags("{ tag_1 }").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{ tag_1 }");

    path.add_tags("{ tag_2 }").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{ tag_1, tag_2 }");

    path.add_tags("{ tag_4, tag_3 }").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{ tag_1, tag_2, tag_3, tag_4 }");

    path.subtract_tags("{ tag_1, tag_3 }").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{ tag_2, tag_4 }");

    path.subtract_tags("{ tag_1 }").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/tfs/{ tag_2, tag_4 }");
}
