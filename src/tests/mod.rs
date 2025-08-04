use std::{error::Error, ffi::OsString, fs, path::PathBuf, process::Command};

use fixtures::{fuse_cleanup, fuse_setup};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{errors::Result_, inodes::QueryInode, unwrap_or};

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

    fuse_cleanup(setup_payload, |setup_payload| {
        let _ = Command::new("mkdir")
            .arg(setup_payload.0.join("tag_1"))
            .arg(setup_payload.0.join("tag_2"))
            .arg(setup_payload.0.join("tag_3"))
            .status()?;

        let _ = Command::new("rmdir")
            .arg(setup_payload.0.join("tag_2"))
            .status()?;

        let _ = Command::new("touch")
            .arg(setup_payload.0.join("{ tag_1 }").join("file_1"))
            .status()?;
        let _ = Command::new("touch")
            .arg(setup_payload.0.join("file_2"))
            .status()?;

        print_tfs(&setup_payload.0.join("{ tag_1 }"))?;
        
        print_delimiter();

        print_tfs(&setup_payload.0)?;

        print_delimiter();

        fs::create_dir(setup_payload.0.join("tag_4"));
        print_tfs(&setup_payload.0)?;

        print_delimiter();

        fs::remove_dir(setup_payload.0.join("tag_4"));
        print_tfs(&setup_payload.0)?;

        print_delimiter();

        fs::remove_dir(setup_payload.0.join("tag_1"));
        print_tfs(&setup_payload.0)?;

        print_delimiter();

        fs::remove_file(setup_payload.0.join("{}").join("file_1"));
        print_tfs(&setup_payload.0)?;

        Ok(())
    }).unwrap();
}

fn print_delimiter() {
    println!("---------"); 
}

fn print_tfs(query_string: &PathBuf) -> Result_<()> {
    for entry in fs::read_dir(query_string)? {
        println!("{}", entry?.path().display());
    }
    Ok(())
}

#[test]
fn test_query_inode() {
    let mut query_inode = QueryInode::try_from(2).unwrap();
    assert_eq!(query_inode.get_id(), 2);
    query_inode = query_inode.get_next();
    assert_eq!(query_inode.get_id(), 3);

    query_inode = QueryInode::try_from(100).unwrap();
    assert_eq!(query_inode.get_id(), 100);
    query_inode = query_inode.get_next();
    assert_eq!(query_inode.get_id(), 2);
}

#[test]
fn test_unwrap_or() {
    let x: Result_<_>  = Ok(1);
    let x = unwrap_or!(x, e, {
        println!("Should not be seen. {e}");
        assert!(false);
        return;
    });
    assert_eq!(x, 1);

    let x = Err("Here I am.");
    _ = unwrap_or!(x, e, {
        println!("Should be seen. {e}");
        return;
    });
    assert!(false);

    let x = Some(1);
    let x = unwrap_or!(x, {
        assert!(false);
        return;
    });
    assert_eq!(x, 1);

    let x = None;
    _ = unwrap_or!(x, {
        return;
    });
    assert!(false);
}

#[test]
fn test_whitespaces() {
    println!(
        "File `{}` with tags \
        `{:?}` does not exist.",
        "goob", "dood"
    );
}
