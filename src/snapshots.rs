use std::{fs::{self, create_dir_all, File}, io::Write, path::PathBuf};

use tracing::{info, instrument};

use crate::{errors::Result_, path_::get_configuration_directory, wrappers::PathExt};

const STAGING_SNAPSHOT_FILENAME: &str = "staging.snap";
const SAFE_SNAPSHOT_FILENAME: &str = "safe.snap";

#[derive(Debug)]
pub struct PersistentSnapshots {
    staging: PathBuf,
    safe: PathBuf
}

impl PersistentSnapshots {
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
        let to_staging = snapshot_directory.join(STAGING_SNAPSHOT_FILENAME);
        let to_safe = snapshot_directory.join(SAFE_SNAPSHOT_FILENAME);
        Ok(Self {
            staging: to_staging,
            safe: to_safe,
        })
    }

    #[instrument(skip_all)]
    pub fn open_staging(&self) -> Result_<File> {
        let staging_snapshot = File::create(&self.staging)?;
        info!("Opened and truncated `{}`.", &self.staging.to_string_lossy());
        Ok(staging_snapshot)
    }

    #[instrument(skip_all)]
    pub fn open_safe(&self) -> Result_<File> {
        let safe_snapshot = File::open(&self.safe)?;
        info!("Opened `{}`.", &self.safe.to_string_lossy());
        Ok(safe_snapshot)
    }

    #[instrument(skip_all)]
    pub fn promote_staging(&self) -> Result_<()> {
        let mut staging_snapshot = File::open(&self.staging)?;
        staging_snapshot.flush();
        info!("Flushed `{}`.", &self.staging.to_string_lossy());
        fs::rename(&self.staging, &self.safe);
        info!("Renamed `{}` to `{}`.",
            &self.staging.to_string_lossy(),
            &self.safe.to_string_lossy());
        Ok(())
    }
}
