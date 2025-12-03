use std::{fs::{self, create_dir_all, File}, io::Write, path::PathBuf};

use tracing::{info, instrument};

use crate::{errors::Result_, path_::get_configuration_directory, wrappers::PathExt};

pub trait TfsSnapshots {
    fn open_staging(&self) -> Result_<File>; 
    fn open_safe(&self) -> Result_<File>;
    fn promote_staging(&self) -> Result_<()>;
}

#[derive(Debug)]
pub struct PersistentSnapshots {
    staging: PathBuf,
    safe: PathBuf
}

// TODO/WIP: Add checksums
impl PersistentSnapshots {
    const STAGING_FILENAME: &str = "staging.snap";
    const SAFE_FILENAME: &str = "safe.snap";

    #[instrument]
    pub fn try_new(location_suffix: &PathBuf) -> Result_<Self> {
        let snapshot_directory = get_configuration_directory()
            .join(location_suffix.__strip_prefix("/"));
        let does_exist = snapshot_directory.try_exists()?;
        if does_exist && !snapshot_directory.is_dir() {
            return Err(format!("`{}` already exists as a non-directory.",
                snapshot_directory.to_string_lossy()).into()); 
        } else if !does_exist {
            create_dir_all(&snapshot_directory)?;
            info!("Created directory `{}`.", snapshot_directory.to_string_lossy());
        }
        let to_staging = snapshot_directory.join(Self::STAGING_FILENAME);
        let to_safe = snapshot_directory.join(Self::SAFE_FILENAME);
        Ok(Self {
            staging: to_staging,
            safe: to_safe,
        })
    }
}

impl TfsSnapshots for PersistentSnapshots {
    #[instrument(skip_all)]
    fn open_staging(&self) -> Result_<File> {
        let staging_snapshot = File::create(&self.staging)?;
        info!("Opened and truncated `{}`.", &self.staging.to_string_lossy());
        Ok(staging_snapshot)
    }

    #[instrument(skip_all)]
    fn open_safe(&self) -> Result_<File> {
        let safe_snapshot = File::open(&self.safe)?;
        info!("Opened `{}`.", &self.safe.to_string_lossy());
        Ok(safe_snapshot)
    }

    #[instrument(skip_all)]
    fn promote_staging(&self) -> Result_<()> {
        let mut staging_snapshot = File::open(&self.staging)?;
        staging_snapshot.flush()?;
        info!("Flushed `{}`.", &self.staging.to_string_lossy());
        fs::rename(&self.staging, &self.safe)?;
        info!("Renamed `{}` to `{}`.",
            &self.staging.to_string_lossy(),
            &self.safe.to_string_lossy());
        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct StubSnapshots;

#[cfg(test)]
impl TfsSnapshots for StubSnapshots {
    fn open_staging(&self) -> Result_<File> {
        Err("No actual file for stub.".into())
    }

    fn open_safe(&self) -> Result_<File> {
        Err("No actual file for stub.".into())
    }

    fn promote_staging(&self) -> Result_<()> {
        Ok(())
    }
}
