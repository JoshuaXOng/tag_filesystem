use std::{error::Error, ffi::OsString, path::PathBuf, process::Command, thread::{self, JoinHandle}};

use fuser::{mount2, MountOption};
use tracing::{info, instrument};
use mount_watcher::{MountWatcher, WatchControl};
use tempfile::tempdir;

use crate::{errors::Result_, filesystem::TagFilesystem};

type FusePayload = (PathBuf, JoinHandle<Result<(), std::io::Error>>);

#[instrument]
pub fn fuse_setup() -> Result<FusePayload, Box<(dyn Error + 'static)>> {
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
            TagFilesystem::new(), &temporary_directory_,
            &[
                MountOption::AutoUnmount,
                MountOption::AllowRoot,
                MountOption::CUSTOM(String::from("-o user"))
            ]
        )
    });

    info!("Waiting for notification that FUSE GFS is mounted.");
    if let Err(e) = watcher_handle.join() {
        Err(format!("{e:?}"))?;
    }

    Ok((temporary_directory, mount_handle))
}

pub fn fuse_cleanup(
    setup_payload: FusePayload,
    to_do: impl FnOnce(&FusePayload) -> Result_<()>
)-> Result_<()> {
    _ = to_do(&setup_payload); 
    fuse_cleanup_(setup_payload)
}

#[instrument]
fn fuse_cleanup_(setup_payload: FusePayload) -> Result_<()> {
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
