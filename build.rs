use std::path::PathBuf;

use capnpc::CompilerCommand;

const CAPNP_SCHEMA_DIRECTORY: &str = "schemas";

fn main() {
    let schemas_directory = PathBuf::from(CAPNP_SCHEMA_DIRECTORY);
    CompilerCommand::new()
        .src_prefix(&schemas_directory)
        .file(schemas_directory.join("filesystem.capnp"))
        .file(schemas_directory.join("file.capnp"))
        .file(schemas_directory.join("tag.capnp"))
        .run()
        .unwrap();
}
