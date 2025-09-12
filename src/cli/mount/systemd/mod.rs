use std::{env::current_exe, fs::canonicalize, path::PathBuf};

use askama::Template;
use clap::Args;

use crate::{cli::ProgramParameters, errors::{AnyError, Result_}};

#[derive(Args, Debug)]
pub struct SystemdParamereters {
    pub mount_path: PathBuf 
}

impl SystemdParamereters {
    pub fn run(&self, _program_arguments: &ProgramParameters) -> Result_<()> {
        // TODO: To implement, something like below.
        // let service_configuration = ServiceTemplate::try_from(self)?
        //     .render()?;
        // let to_configuration = PathBuf::from(SYSTEMD_SERVICE_DIRECTORY)
        //     .join(SERVICE_FILE_NAME);
        // if program_arguments.dry {
        //     info!("Would have written `{}` to `{}`.", service_configuration,
        //         to_configuration.to_string_lossy());
        // } else {
        //     File::create(&to_configuration)?
        //         .write_all(service_configuration.as_bytes())?;
        //     info!("Wrote `{}` to `{}`.", service_configuration,
        //         to_configuration.to_string_lossy());
        // }
        // Ok(())
        unimplemented!()
    }
}

pub const SYSTEMD_SERVICE_DIRECTORY: &str = "/etc/systemd/system";
pub const SERVICE_FILE_NAME: &str = "tag_filesystem.service";

#[derive(Template)]
#[template(path = "tag_filesystem.service.j2")] 
pub struct ServiceTemplate {
    tfs_binary_path: String,
    mount_path: String,
    mount_user: String,
}

impl TryFrom<&SystemdParamereters> for ServiceTemplate {
    type Error = AnyError;

    fn try_from(value: &SystemdParamereters) -> Result<Self, Self::Error> {
        Ok(Self {
            tfs_binary_path: canonicalize(current_exe()?)?
                .to_string_lossy()
                .into_owned(),
            mount_path: canonicalize(&value.mount_path)?
                .to_string_lossy()
                .into_owned(),
            mount_user: users::get_current_username()
                .ok_or("Don't got no username.")?
                .to_string_lossy()
                .into_owned()
        })
    }
}
