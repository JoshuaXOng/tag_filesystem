use std::{error::Error, path::PathBuf, process::Command, thread::{self, JoinHandle}};

use fuser::{mount2, MountOption};
use tracing::{info, instrument};
use mount_watcher::{MountWatcher, WatchControl};
use tempfile::tempdir;

use crate::{errors::Result_, filesystem::GraphFilesystem};

#[instrument]
pub fn fuse_setup() -> Result<
    (PathBuf, JoinHandle<Result<(), std::io::Error>>),
    Box<(dyn Error + 'static)>
> {
    let x = tempdir()?;
    let temporary_directory = x
        .path()
        .to_path_buf();
    info!("Setting up FUSE mount, created temp. dir. `{temporary_directory:?}`.");

    let temporary_directory_ = temporary_directory.clone();
    let watcher_handle = MountWatcher::new(move |mount_events| {
        let temporary_directory_ = match temporary_directory_.to_str() {
            Some(x) => x,
            None => return WatchControl::Stop,
        };

        let have_mounted = mount_events.mounted.iter().any(|mount_event| {
            if mount_event.fs_type == "fuse"
            && mount_event.mount_point == temporary_directory_
            { true }
            else { false }
        });
        if have_mounted { WatchControl::Stop }
        else { WatchControl::Continue }
    })?;

    let temporary_directory_ = temporary_directory.clone();
    let mount_handle = thread::spawn(move || {
        info!("Mounting at `{temporary_directory_:?}`.");
        mount2(
            GraphFilesystem::new_test(),&temporary_directory_,
            &[MountOption::CUSTOM(String::from("-o user"))]
        )
    });

    info!("Waiting for notification that FUSE GFS is mounted.");
    if let Err(e) = watcher_handle.join() {
        Err(format!("{e:?}"))?;
    }

    Ok((temporary_directory, mount_handle))
}

#[instrument]
pub fn fuse_cleanup(setup_payload: (PathBuf, JoinHandle<Result<(), std::io::Error>>)) ->
Result_<()> {
    info!("Running cleanup for FUSE FS, unmounting at `{:?}`.", setup_payload.0);
    let umount_process = Command::new("umount")
        .arg(setup_payload.0)
        .status()?;
    assert!(umount_process.success());
    
    info!("Waiting for FUSE mount invocation to return.");
    match setup_payload.1.join() {
        Ok(x) => x?,
        Err(e) => Err(format!("{e:?}"))?,
    };

    Ok(())
}

impl GraphFilesystem {
    #[instrument]
    pub fn new_test() -> Self {
        let mut graph_filesystem = GraphFilesystem::new();

        let tag_1 = graph_filesystem.create_tag("tag_1");
        let tag_2 = graph_filesystem.create_tag("tag_2");
        let tag_3 = graph_filesystem.create_tag("tag_2");

        let file_1 = graph_filesystem.create_file("file_1");
        let file_2 = graph_filesystem.create_file("file_2");
        let file_3 = graph_filesystem.create_file("file_3");

        graph_filesystem.add_tag_to_file(
            file_1.ino, tag_1.ino
        ).unwrap();

        graph_filesystem
    }
}
