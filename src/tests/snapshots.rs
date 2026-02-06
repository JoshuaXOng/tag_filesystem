use std::io::{BufReader, Read, Write};

use tempfile::tempdir;

use crate::{
    errors::ResultBtAny,
    snapshots::{PersistentSnapshots, TfsSnapshots},
    tests::tracing::setup_tracing,
};

#[test]
fn running_normal_snapshot_cycle() -> ResultBtAny<()> {
    setup_tracing();

    let temporary_directory = tempdir()?;
    let persistent_snapshots =
        PersistentSnapshots::try_new(&temporary_directory.path().to_path_buf())?;

    persistent_snapshots.open_safe().unwrap_err();

    for (payload_index, snapshot_payload) in ["test1234abcd", "5678efgh"].into_iter().enumerate() {
        let mut staging_snapshot = persistent_snapshots.create_staging()?;
        staging_snapshot.write_all(snapshot_payload.as_bytes());

        let is_first_iteration = payload_index == 0;
        assert_eq!(
            is_first_iteration,
            persistent_snapshots.open_safe().is_err()
        );

        persistent_snapshots.promote_staging()?;
        let mut safe_snapshot = persistent_snapshots.open_safe()?;
        let mut safe_contents = String::new();
        safe_snapshot.read_to_string(&mut safe_contents);
        assert_eq!(safe_contents, snapshot_payload);
    }

    Ok(())
}
