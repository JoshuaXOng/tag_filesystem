use std::{fs::{self, create_dir_all, remove_file, File, OpenOptions}, io::{Read, Seek, SeekFrom,
    Write}, path::PathBuf, time::SystemTime};

use tracing::{info, instrument, warn};

use crate::{errors::Result_, inodes::FileInode, path_::get_configuration_directory, wrappers::PathExt};

pub trait TfsStorage {
    fn get_file_size(&self, file_inode: &FileInode) -> Result_<u64>;
    fn get_last_accessed(&self, file_inode: &FileInode) -> Result_<SystemTime>;
    fn get_last_modified(&self, file_inode: &FileInode) -> Result_<SystemTime>;
    fn get_when_created(&self, file_inode: &FileInode) -> Result_<SystemTime>;
    fn read(&self, file_inode: &FileInode, start_position: u64,
        read_amount: usize) -> Result_<Vec<u8>>;
    fn write(&mut self, file_inode: &FileInode, start_position: u64,
        to_write: &[u8]) -> Result_<()>;
    fn delete(&self, file_inode: &FileInode) -> Result_<()>;
}

#[derive(Debug)]
pub struct DelegateStorage {
    root: PathBuf
}

impl DelegateStorage {
    pub const DELEGATE_DIRECTORY_NAME: &str = "delegate_storage";

    #[instrument]
    pub fn try_new(location_suffix: &PathBuf) -> Result_<Self> {
        let delegate_directory = Self::get_delegate_directory(location_suffix);
        let does_exist = delegate_directory.try_exists()?;
        if does_exist && !delegate_directory.is_dir() {
            if !delegate_directory.is_dir() {
                return Err(format!("Delgate storage root needs a dir not \
                    a file `{}`.", delegate_directory.to_string_lossy()).into());
            }
            warn!("Delegate storage already exists at `{}`, re-using it.",
                delegate_directory.to_string_lossy());
        } else if !does_exist {
            create_dir_all(&delegate_directory)?;
            // TODO: tracing should be before side-effect in rust
            info!("Creating directories on the way to `{}`.",
                delegate_directory.to_string_lossy());
        }
        Ok(Self { root: delegate_directory })
    }

    fn get_location_prefix() -> PathBuf {
        get_configuration_directory().join(Self::DELEGATE_DIRECTORY_NAME)
    }

    pub fn get_delegate_directory(location_suffix: &PathBuf) -> PathBuf {
        let mut delegate_directory = Self::get_location_prefix();
        delegate_directory.push(location_suffix.__strip_prefix("/"));
        delegate_directory
    }

    fn get_delegate_path(&self, file_inode: &FileInode) -> PathBuf {
        let mut delegate_path = self.root.clone(); 
        delegate_path.push(file_inode.get_id().to_string());
        delegate_path
    }
}

impl TfsStorage for DelegateStorage {
    fn get_file_size(&self, file_inode: &FileInode) -> Result_<u64> {
        Ok(fs::metadata(self.get_delegate_path(file_inode))?
            .len())
    }

    fn get_last_accessed(&self, file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(fs::metadata(self.get_delegate_path(file_inode))?
            .accessed()?)
    }

    fn get_last_modified(&self, file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(fs::metadata(self.get_delegate_path(file_inode))?
            .modified()?)
    }

    fn get_when_created(&self, file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(fs::metadata(self.get_delegate_path(file_inode))?
            .created()?)
    }

    fn read(&self, file_inode: &FileInode, start_position: u64, read_amount: usize)
    -> Result_<Vec<u8>> {
        let delegate_path = self.get_delegate_path(file_inode);
        let mut delegate_file = File::open(delegate_path)?;
        delegate_file.seek(SeekFrom::Start(start_position))?;
        let mut file_contents = vec![0u8; read_amount];
        let actual_amount = delegate_file.read(&mut file_contents)?;
        file_contents.truncate(actual_amount);
        Ok(file_contents)
    }

    fn write(&mut self, file_inode: &FileInode, start_position: u64, to_write: &[u8])
    -> Result_<()> {
        let delegate_path = self.get_delegate_path(file_inode);
        let mut delegate_file = OpenOptions::new()
            .create(true)
            .read(true) 
            .write(true)
            .open(&delegate_path)?;
        delegate_file.seek(SeekFrom::Start(start_position))?;
        delegate_file.write_all(to_write)?;
        Ok(())
    }

    fn delete(&self, file_inode: &FileInode) -> Result_<()> {
        let delegate_path = self.get_delegate_path(file_inode);
        remove_file(delegate_path)
            .map_err(Into::into)
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct StubStorage;

#[cfg(test)]
impl TfsStorage for StubStorage {
    fn get_file_size(&self, _file_inode: &FileInode) -> Result_<u64> {
        Ok(0)
    }

    fn get_last_accessed(&self, _file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn get_last_modified(&self, _file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn get_when_created(&self, _file_inode: &FileInode) -> Result_<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn read(&self, _file_inode: &FileInode, _start_position: u64, _read_amount: usize)
    -> Result_<Vec<u8>> {
        Ok(vec![])
    }

    fn write(&mut self, _file_inode: &FileInode, _start_position: u64, _to_write: &[u8])
    -> Result_<()> {
        Ok(())
    }

    fn delete(&self, _file_inode: &FileInode) -> Result_<()> {
        Ok(())
    }
}
