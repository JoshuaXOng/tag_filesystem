use std::{env::current_exe, fs::canonicalize, path::Path};

use askama::Template;
use clap::Args;

use crate::{cli::{mount::MountParameters, ProgramParameters}, errors::ResultBtAny};

#[derive(Args, Debug)]
pub struct SystemdParamereters;

impl SystemdParamereters {
    pub fn run(&self, _program_arguments: &ProgramParameters,
        _mount_arguments: &MountParameters) -> ResultBtAny<()>
    {
        // TODO: To implement, something like below.
        // let service_configuration = ServiceTemplate::try_new(_mount_arguments.mount_path)
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

impl ServiceTemplate {
    pub fn try_new(mount_path: &Path) -> ResultBtAny<Self> {
        Ok(ServiceTemplate {
            tfs_binary_path: canonicalize(current_exe()?)?
                .to_string_lossy()
                .into_owned(),
            mount_path: canonicalize(mount_path)?
                .to_string_lossy()
                .into_owned(),
            mount_user: users::get_current_username()
                .ok_or("Don't got no username.")?
                .to_string_lossy()
                .into_owned()
        })
    }
}
