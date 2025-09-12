use std::{env::current_exe, fs::canonicalize, path::PathBuf};

use askama::Template;
use clap::Parser;
use tempfile::tempdir;

use crate::{cli::{mount::systemd::{ServiceTemplate, SystemdParamereters}, tags::change::{ChangeParameters, ChangeTag}}, path_::PathBufExt, tests::tracing_::setup_tracing};

#[test]
fn parsing_changing_tags() {
    let change_arguments = ChangeParameters::parse_from(["ct", "tag_1"]);
    assert_eq!(
        change_arguments,
        ChangeParameters {
            are_negated: false,
            tags: vec![ChangeTag::builder()
                .name("tag_1")
                .build()]
        });

    let change_arguments = ChangeParameters::parse_from(["ct", "~tag_1"]);
    assert_eq!(
        change_arguments,
        ChangeParameters {
            are_negated: false,
            tags: vec![ChangeTag::builder()
                .is_negated(true)
                .name("tag_1")
                .build()]
        });

    let change_arguments = ChangeParameters::parse_from(["ct", "-n", "tag_1", "tag_2"]);
    assert_eq!(
        change_arguments,
        ChangeParameters {
            are_negated: true,
            tags: vec![ChangeTag::builder()
                .name("tag_1")
                .build(),
                ChangeTag::builder()
                    .name("tag_2")
                    .build()]
        });

    let change_arguments = ChangeParameters::parse_from(["ct", "-n", "tag_1", "~tag_2"]);
    assert_eq!(
        change_arguments,
        ChangeParameters {
            are_negated: true,
            tags: vec![ChangeTag::builder()
                .name("tag_1")
                .build(),
                ChangeTag::builder()
                    .is_negated(true)
                    .name("tag_2")
                    .build()]
        });

    let mut path = PathBuf::new()
        .join("/tmp/ct/{ tag_8, tag_2 }");
    println!("{:?}", path);

    let tags = change_arguments.tags;

    println!("{:?}", path);
}

// TODO: Rewrite tests.
#[test]
fn systemd_unit_file_rendering() {
    setup_tracing();

    let temporary_directory = tempdir().unwrap();
    let arguments = SystemdParamereters { 
        mount_path: temporary_directory.path()
            .to_path_buf()
    };
    let service_configuration = ServiceTemplate::try_from(&arguments)
        .unwrap()
        .render()
        .unwrap();
    assert_eq!(service_configuration, 
        format!(indoc::indoc!(
            "[Unit]
            Description=Tag Filesystem
            [Service]
            Type=simple
            ExecStart={to_binary} {mount_path}
            ExecStop={to_binary} -u {mount_path}
            Restart=always
            RestartSec=5
            User=jxo
            
            DeviceAllow=/dev/fuse rw
            [Install]
            WantedBy=multi-user.target"), 
            to_binary=canonicalize(current_exe().unwrap())
                .unwrap()
                .display()
                .to_string(),
            mount_path=arguments.mount_path.display()
                .to_string()));
}
