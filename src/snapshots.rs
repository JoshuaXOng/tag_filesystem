use std::{fs::{self, create_dir_all, File}, io::Write, path::{Path, PathBuf}};

use derive_more::{Display, Error};
use drums::Backtrace;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, instrument};

use crate::{errors::{AnyError, ResultBt, ResultBtAny},
    path::get_configuration_directory, wrappers::PathExt};

pub trait TfsSnapshots {
    fn open_safe(&self) -> ResultBt<File, OpenError>;
    fn create_staging(&self) -> ResultBtAny<File>; 
    fn promote_staging(&self) -> ResultBtAny<()>;
}

#[derive(Debug, Display, Error, Backtrace)]
#[display("SnapshotError::{_variant}")]
#[bt_from(AnyError, std::io::Error)]
pub enum OpenError {
    Checksum(AnyError),
    Other(AnyError)
}

impl From<AnyError> for OpenError {
    fn from(value: AnyError) -> Self {
        Self::Other(value)
    }
}

impl From<std::io::Error> for OpenError {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.into())
    }
}

#[derive(Debug)]
pub struct PersistentSnapshots {
    root: PathBuf
}

impl PersistentSnapshots {
    const SNAPSHOT_DIRECTORY_NAME: &str = "snapshots";
    const POINTERS_FILENAME: &str = "pointers.json";
    const SNAPSHOT_FILENAME: &str = "tfs.snapshot";
    const SHA512_FILENAME: &str = "tfs.snapshot.sha256";

    #[instrument]
    pub fn try_new(location_suffix: &PathBuf) -> ResultBtAny<Self> {
        let snapshot_directory = get_configuration_directory()
            .join(Self::SNAPSHOT_DIRECTORY_NAME)
            .join(location_suffix.__strip_prefix("/"));
        let does_exist = snapshot_directory.try_exists()?;
        if does_exist && !snapshot_directory.is_dir() {
            return Err(format!("`{}` already exists as a non-directory.",
                snapshot_directory.to_string_lossy()).into()); 
        } else if !does_exist {
            create_dir_all(&snapshot_directory)?;
            info!("Created directory `{}`.", snapshot_directory.to_string_lossy());
        }

        let _self = Self {
            root: snapshot_directory
        };

        let pointers_path = _self.get_pointers_path();
        let does_exist = pointers_path.try_exists()?;
        if !does_exist {
            fs::write(
                &pointers_path,
                serde_json::to_string(&SnapshotPointers {
                    snapshot: None,
                    sha256: None
                })?)?;
            info!("Wrote to pointers file `{}`.", pointers_path.to_string_lossy());
        }

        Ok(_self)
    }

    fn get_pointers_path(&self) -> PathBuf {
        self.root.join(Self::POINTERS_FILENAME)
    }

    fn get_snapshot_pointers(&self) -> ResultBtAny<SnapshotPointers> {
        let to_read = self.get_pointers_path();
        let as_json = &fs::read(&to_read)?;
        info!("Read snapshot pointers file, `{}`.", to_read.to_string_lossy());
        Ok(serde_json::from_slice(as_json)?)
    }

    fn get_opposite_snapshot_path(&self) -> ResultBtAny<PathBuf> {
        let snapshot_pointers = self.get_snapshot_pointers()?;
        let to_snapshot = if let Some(mut to_snapshot) = snapshot_pointers.snapshot {
            PointerChoice::switch_extension(&mut to_snapshot)?;
            to_snapshot
        } else {
            self.get_snapshot_path(&PointerChoice::Blue)
        };
        Ok(to_snapshot)
    }

    fn get_opposite_sha256_path(&self) -> ResultBtAny<PathBuf> {
        let snapshot_pointers = self.get_snapshot_pointers()?;
        let to_sha256 = if let Some(mut to_sha256) = snapshot_pointers.sha256 {
            PointerChoice::switch_extension(&mut to_sha256)?;
            to_sha256
        } else {
            self.get_sha256_path(&PointerChoice::Blue)
        };
        Ok(to_sha256)
    }

    fn get_snapshot_path(&self, pointer_choice: &PointerChoice) -> PathBuf {
        let path = self.root.join(Self::SNAPSHOT_FILENAME);
        PathBuf::from(format!(
            "{}.{}",
            path.to_string_lossy(), pointer_choice.get_extension()))
    }

    fn get_sha256_path(&self, pointer_choice: &PointerChoice) -> PathBuf {
        let path = self.root.join(Self::SHA512_FILENAME);
        PathBuf::from(format!(
            "{}.{}",
            path.to_string_lossy(), pointer_choice.get_extension()))
    }

    fn get_sha256_from(file_path: &Path, result_container: &mut Vec<u8>)
    -> ResultBtAny<()> {
        let sha256_digest = Sha256::digest(fs::read(&file_path)?);
        result_container.write_all(sha256_digest.as_slice())?;
        info!("Wrote SHA-256 to container.");
        Ok(())
    }
}

impl TfsSnapshots for PersistentSnapshots {
    #[instrument(skip_all)]
    fn open_safe(&self) -> ResultBt<File, OpenError> {
        let snapshot_pointers = self.get_snapshot_pointers()?;
        let to_snapshot = snapshot_pointers.get_snapshot()?;
        
        let safe_snapshot = File::open(&to_snapshot)?;
        info!("Opened `{}`.", to_snapshot.to_string_lossy());
        
        let mut computed_sha256 = vec![];
        Self::get_sha256_from(&to_snapshot, &mut computed_sha256)?;
        let to_sha256 = snapshot_pointers.get_sha256()?;
        let stored_sha256 = fs::read(&to_sha256)?;
        info!("Read SHA-256 file `{}`.", to_sha256.to_string_lossy());
        let did_get_malformed = computed_sha256 != stored_sha256;
        if did_get_malformed {
            return Err(OpenError::Checksum("Computed SHA-256 of safe \
                snapshot is not equal to the stored SHA-256 of the snapshot.".into())
                .into());
        }

        Ok(safe_snapshot)
    }

    #[instrument(skip_all)]
    fn create_staging(&self) -> ResultBtAny<File> {
        let to_snapshot = self.get_opposite_snapshot_path()?;
        let staging_snapshot = File::create(&to_snapshot)?;
        info!("Opened and truncated `{}`.", to_snapshot.to_string_lossy());
        Ok(staging_snapshot)
    }

    #[instrument(skip_all)]
    fn promote_staging(&self) -> ResultBtAny<()> {
        let to_snapshot = self.get_opposite_snapshot_path()?;
        let mut staging_snapshot = File::open(&to_snapshot)?;
        staging_snapshot.flush()?;
        info!("Flushed `{}`.", to_snapshot.to_string_lossy());

        let to_sha256 = self.get_opposite_sha256_path()?;
        let mut staging_sha256 = File::create(&to_sha256)?;
        let mut computed_sha256 = vec![];
        Self::get_sha256_from(&to_snapshot, &mut computed_sha256)?;
        staging_sha256.write_all(&computed_sha256)?;
        staging_sha256.flush()?;
        info!("Flushed `{}`.", to_sha256.to_string_lossy());

        let to_pointers = PathBuf::from(format!(
            "{}.{}",
            self.get_pointers_path()
                .to_string_lossy(),
            "staging"));
        fs::write(
            &to_pointers,
            serde_json::to_string(&SnapshotPointers {
                snapshot: Some(to_snapshot),
                sha256: Some(to_sha256)
            })?)?;
        info!("Wrote to file `{}`.", &to_pointers.to_string_lossy());

        fs::rename(&to_pointers, self.get_pointers_path())?;
        info!("Renamed `{}` to `{}`.",
            to_pointers.to_string_lossy(),
            self.get_pointers_path().to_string_lossy());
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct SnapshotPointers {
    snapshot: Option<PathBuf>,
    sha256: Option<PathBuf>
}

impl SnapshotPointers {
    fn get_snapshot(&self) -> ResultBtAny<PathBuf> {
        self.snapshot
            .clone()
            .ok_or("Pointer to snapshot not yet initialized.".into())
    }

    fn get_sha256(&self) -> ResultBtAny<PathBuf> {
        self.sha256
            .clone()
            .ok_or("Pointer to snapshot checksum not yet initialized.".into())
    }
}

enum PointerChoice {
    Blue,
    Green
}

impl PointerChoice {
    const BLUE_EXTENSION: &str = "blue";
    const GREEN_EXTENSION: &str = "green";

    fn get_extension(&self) -> &str {
        match self {
            PointerChoice::Blue => Self::BLUE_EXTENSION,
            PointerChoice::Green => Self::GREEN_EXTENSION
        }
    }

    fn switch_extension(path: &mut PathBuf) -> ResultBtAny<()> {
        let other_extension = match path.extension()
            .map(|extension| extension.to_str().unwrap_or(""))
        {
            Some(PointerChoice::BLUE_EXTENSION) => PointerChoice::Green,
            Some(PointerChoice::GREEN_EXTENSION) => PointerChoice::Blue,
            None => return Err(format!(
                "Can't switch path w/o extension `{}`.",
                path.to_string_lossy()).into()),
            Some(_) => return Err(format!(
                "Can't switch path's `{}` extension.",
                path.to_string_lossy()).into())
        };

        path.set_extension(other_extension.get_extension());

        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct StubSnapshots;

#[cfg(test)]
impl TfsSnapshots for StubSnapshots {
    fn open_safe(&self) -> ResultBt<File, OpenError> {
        return Err(OpenError::Checksum("No actual file for stub.".into())
            .into());
    }

    fn create_staging(&self) -> ResultBtAny<File> {
        Err("No actual file for stub.".into())
    }

    fn promote_staging(&self) -> ResultBtAny<()> {
        Ok(())
    }
}
