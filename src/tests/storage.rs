use std::path::PathBuf;

use crate::{path_::get_configuration_directory, storage::DelegateStorage};

#[test]
fn handling_double_slash() {
    let suspect_path = PathBuf::from("//etc/dobothleading/getremoved");
    let delegate_directory = DelegateStorage::get_delegate_directory(&suspect_path); 
    assert_eq!(delegate_directory, get_configuration_directory()
        .join(DelegateStorage::DELEGATE_DIRECTORY_NAME)
        .join("etc/dobothleading/getremoved"))
}
