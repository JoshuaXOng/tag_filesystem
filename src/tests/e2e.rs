use std::{env::{current_dir, set_current_dir}, error::Error, ffi::{OsStr, OsString}, fs::{self,
    rename, File, OpenOptions}, io::{stdout, Write}, path::PathBuf, process::{self, Command,
    ExitStatus, Stdio}};

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{errors::Result_, tests::{fixtures::with_tfs_mount, tracing_::setup_tracing},
    wrappers::VecWrapper};

#[test]
fn listing_files_and_tags() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("{ tag_1 }").join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("{ tag_1, tag_2 }").join("file_3"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_1\ntag_2\n");
        let output = cmd("ls").arg(mount_directory.join("{}"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_1\ntag_2\n");

        let output = cmd("ls").arg(mount_directory.join("file_1"))
            .run_and_log()?;
        assert_eq!(output, mount_directory.join("file_1").to_string_lossy() + "\n");

        let output = cmd("ls").arg(mount_directory.join("tag_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1 }"))
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_1\ntag_2\n");

        let output = cmd("ls").arg(mount_directory.join("{ tag_2 }"))
            .run_and_log()?;
        assert_eq!(output, "tag_1\ntag_2\n");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1, tag_2 }"))
            .run_and_log()?;
        assert_eq!(output, "file_3\ntag_1\ntag_2\n");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1 }").join("file_2"))
            .run_and_log()?;
        assert_eq!(output,
            mount_directory.join("{ tag_1 }").join("file_2").to_string_lossy() + "\n");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1 }").join("tag_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        Ok(())
    }).unwrap();
}

#[test]
fn listing_namespace_to_show_neighbour_tags() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("touch").arg(mount_directory.join("{ tag_1 }").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("touch").arg(mount_directory.join("{ tag_1, tag_2 }").join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_1\ntag_2\n");

        Ok(())
    }).unwrap();
}

#[test]
fn creating_files() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("touch")
            .arg(mount_directory.join("file_1"))
            .arg(mount_directory.join("file_2"))
            .arg(mount_directory.join("file_3"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\nfile_2\nfile_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn creating_duplicate_files() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("touch")
            .arg(mount_directory.join("file_1"))
            .arg(mount_directory.join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch")
            .arg(mount_directory.join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\nfile_2\n");

        Ok(())
    }).unwrap();
}

#[test]
fn creating_tags() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .arg(mount_directory.join("tag_3"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "tag_1\ntag_2\ntag_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn creating_duplicate_tags() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .run_and_log()?;
        assert_eq!(output, "");
        cmd("mkdir")
            .arg(mount_directory.join("tag_2"))
            .run_and_log()
            .expect_err("To have already created `tag_2`.");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "tag_1\ntag_2\n");

        Ok(())
    }).unwrap();
}

#[test]
fn writing_and_reading_to_files() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("touch").arg(mount_directory.join("{}").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let mut echo_into = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(mount_directory.join("{}").join("file_1"))?;
        let output = cmd("echo").arg("abcdefghij")
            .stdout(echo_into)
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("cat").arg(mount_directory.join("{}").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "abcdefghij\n");

        Ok(())
    }).unwrap();
}

#[test]
fn removing_file() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("touch")
            .arg(mount_directory.join("file_1"))
            .arg(mount_directory.join("file_2"))
            .arg(mount_directory.join("file_3"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("rm").arg(mount_directory.join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\nfile_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn removing_nonexistent_file() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("touch")
            .arg(mount_directory.join("file_1"))
            .arg(mount_directory.join("file_2"))
            .arg(mount_directory.join("file_3"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("rm").arg(mount_directory.join("file_4"))
            .run_and_log()
            .expect_err("To not have created `file_4`.");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\nfile_2\nfile_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn removing_tags() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .arg(mount_directory.join("tag_3"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("rmdir").arg(mount_directory.join("tag_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "tag_1\ntag_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn removing_nonexistent_tag() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .arg(mount_directory.join("tag_3"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("rmdir").arg(mount_directory.join("tag_4"))
            .run_and_log()
            .expect_err("To not have created `tag_4`.");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "tag_1\ntag_2\ntag_3\n");

        Ok(())
    }).unwrap();
}

#[test]
fn doing_random_chained_interactions() {
    setup_tracing();

    with_tfs_mount(|mount_directory| {
        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_1"))
            .arg(mount_directory.join("tag_2"))
            .arg(mount_directory.join("tag_3"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("rmdir").arg(mount_directory.join("tag_2"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("{ tag_1 }").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("file_2"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory.join("{ tag_1 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_1\n");
        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_1\ntag_3\n");

        let output = cmd("mkdir").arg(mount_directory.join("tag_4"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_1\ntag_3\ntag_4\n");

        let output = cmd("rmdir").arg(mount_directory.join("tag_4"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_1\ntag_3\n");

        let output = cmd("rmdir").arg(mount_directory.join("tag_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_1\nfile_2\ntag_3\n");

        let output = cmd("rm").arg(mount_directory.join("{}").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\n");

        let output = cmd("touch").arg(mount_directory.join("{ tag_3 }").join("file_4"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\n");
        let original_directory = current_dir()?;
        set_current_dir(mount_directory.join("{ tag_3 }"));
        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\n");
        set_current_dir(&original_directory);

        let output = cmd("mkdir")
            .arg(mount_directory.join("tag_4"))
            .arg(mount_directory.join("tag_5"))
            .arg(mount_directory.join("tag_6"))
            .run_and_log()?;
        assert_eq!(output, "");
        let output = cmd("touch").arg(mount_directory.join("{ tag_3, tag_4 }").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\ntag_4\ntag_5\ntag_6\n");
        let output = cmd("ls").arg(mount_directory.join("{ tag_3 }"))
            .run_and_log()?;
        assert_eq!(output, "file_4\ntag_3\ntag_4\n");
        let output = cmd("ls").arg(mount_directory.join("{ tag_3, tag_4 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_3\ntag_4\n");
        let output = cmd("ls").arg(mount_directory.join("{}"))
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\ntag_4\n");

        let output = cmd("mv")
            .arg(mount_directory.join("{ tag_3, tag_4 }").join("file_1"))
            .arg(mount_directory.join("{ tag_3, tag_4, tag_5 }").join("file_1"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory.join("{ tag_3, tag_4 }"))
            .run_and_log()?;
        assert_eq!(output, "tag_3\ntag_4\ntag_5\n");
        let output = cmd("ls").arg(mount_directory.join("{ tag_3, tag_4, tag_5 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_3\ntag_4\ntag_5\n");

        let output = cmd("mv")
            .arg(mount_directory.join("tag_4"))
            .arg(mount_directory.join("tag_44"))
            .run_and_log()?;
        assert_eq!(output, "");

        let output = cmd("ls").arg(mount_directory)
            .run_and_log()?;
        assert_eq!(output, "file_2\ntag_3\ntag_44\ntag_5\ntag_6\n");
        let output = cmd("ls").arg(mount_directory.join("{ tag_3, tag_44, tag_5 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_3\ntag_44\ntag_5\n");
        cmd("ls").arg(mount_directory.join("{ tag_3, tag_4, tag_5 }"))
            .run_and_log()
            .expect_err("No file with such tags.");
        let output = cmd("ls").arg(mount_directory.join("{ tag_3, tag_44, tag_5 }"))
            .run_and_log()?;
        assert_eq!(output, "file_1\ntag_3\ntag_44\ntag_5\n");

        Ok(())
    }).unwrap();
}

trait CommandExt {
    fn run_and_log(&mut self) -> Result_<String>;
}

impl CommandExt for Command {
    fn run_and_log(&mut self) -> Result_<String> {
        println!("> {} {}", 
            self.get_program().to_string_lossy(),
            VecWrapper(self.get_args()
                .map(|arg| arg.to_string_lossy())
                .collect()));
        self.stderr(stdout());
        let command_output = self.output()?;
        let stdout_stderr = str::from_utf8(&command_output.stdout)
            .unwrap_or("Output is invalid UTF-8")
            .to_string();
        print!("{}", stdout_stderr);
        let output_status = command_output.status;
        if !output_status.success() {
            return Err(format!(
                "Exitted with code `{:?}`",
                output_status.code())
                .into());
        }
        Ok(stdout_stderr)
    }
}

fn cmd(program: &str) -> Command {
    Command::new(program)
}
