use std::fs;

use fixtures::{fuse_cleanup, fuse_setup};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod fixtures;

#[test]
fn entries_query() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .init();
    
    let setup_payload = fuse_setup().unwrap();

    for entry in fs::read_dir(&setup_payload.0.join("{ tag_1 }")).unwrap() {
        println!("{}", entry.unwrap().path().display());
    }

    for entry in fs::read_dir(&setup_payload.0).unwrap() {
        println!("{}", entry.unwrap().path().display());
    }

    fuse_cleanup(setup_payload).unwrap();
}
