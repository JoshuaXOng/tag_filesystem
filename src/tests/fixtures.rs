use std::{error::Error, ffi::OsString, path::PathBuf, process::Command,
    thread::{self, JoinHandle}};

use fuser::{mount2, MountOption};
use tracing::{info, instrument};
use mount_watcher::{MountWatcher, WatchControl};
use tempfile::tempdir;

use crate::{errors::{AnyError, ResultBtAny}, filesystem::TagFilesystem};

pub fn with_tfs_mount(to_do: impl FnOnce(&PathBuf) -> ResultBtAny<()>) -> ResultBtAny<()> {
    let expectation = "Test setup code should work."; 
    let setup_payload = fuse_setup()
        .expect(expectation);
    let test_result = to_do(&setup_payload.0); 
    fuse_cleanup(setup_payload)
        .expect(expectation);
    test_result
        .unwrap();
    Ok(())
}

type FusePayload = (PathBuf, JoinHandle<ResultBtAny<()>>);

#[instrument]
fn fuse_setup() -> Result<FusePayload, Box<(dyn Error + 'static)>> {
    let temporary_directory = tempdir()?;
    let temporary_directory = temporary_directory
        .path()
        .to_path_buf();
    info!("Setting up FUSE mount, created temp. dir. `{temporary_directory:?}`.");

    let temporary_directory_ = temporary_directory.clone();
    let watcher_handle = MountWatcher::new(move |mount_events| {
        let temporary_directory_ = match temporary_directory_.to_str() {
            Some(directory) => directory,
            None => return WatchControl::Stop,
        };

        let have_mounted = mount_events.mounted.iter().any(|mount_event| {
            let is_fuse_mount = mount_event.fs_type == "fuse";
            let is_temporary_directory = mount_event.mount_point == temporary_directory_;
            is_fuse_mount && is_temporary_directory 
        });
        if have_mounted { WatchControl::Stop }
        else { WatchControl::Continue }
    })?;

    let temporary_directory_ = temporary_directory.clone();
    let mount_handle: JoinHandle<ResultBtAny<()>> = thread::spawn(move || {
        info!("Mounting at `{temporary_directory_:?}`.");
        Ok(mount2(
            TagFilesystem::try_new(&temporary_directory_)?,
            &temporary_directory_,
            &[MountOption::AutoUnmount, MountOption::AllowRoot]
        )?)
    });

    info!("Waiting for notification that FUSE GFS is mounted.");
    if let Err(e) = watcher_handle.join() {
        Err(format!("{e:?}"))?;
    }

    Ok((temporary_directory, mount_handle))
}

#[instrument]
fn fuse_cleanup((mount_directory, mount_handle): FusePayload) -> ResultBtAny<()> {
    info!("Running cleanup for FUSE FS, unmounting at `{:?}`.",
        mount_directory);
    let umount_process = Command::new("umount")
        .arg(mount_directory)
        .status()?;
    assert!(umount_process.success());
  
    info!("Waiting for FUSE mount invocation to return.");
    match mount_handle.join() {
        Ok(mount_result) => mount_result?,
        Err(e) => Err(format!("{e:?}"))?,
    };

    Ok(())
}
