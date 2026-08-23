use crate::{backup, state::Store};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const USER_BACKUP_DIR: &str = "backups";
const VALIDATION_ROOT: &str = "recovery-validation";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupPreview {
    candidate_id: String,
    validation: RecoveryBackupValidation,
    work_item_count: Option<usize>,
    has_active_item: Option<bool>,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum RecoveryBackupValidation {
    Usable,
    Unavailable,
}

pub fn validate_recovery_backup(app_data_dir: &Path, candidate_id: &str) -> Result<(usize, bool)> {
    let candidate = resolve_candidate(app_data_dir, candidate_id)?;
    let validation_root = app_data_dir.join(VALIDATION_ROOT);
    fs::create_dir_all(&validation_root).context("create recovery validation root")?;
    let workspace = validation_root.join(format!("candidate-{}", unique_stamp()?));
    fs::create_dir(&workspace).context("create isolated recovery validation workspace")?;

    let result = (|| {
        let backup_dir = workspace.join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir).context("create isolated backup directory")?;
        let staged_candidate = backup_dir.join(candidate_id);
        fs::copy(&candidate, &staged_candidate).context("stage managed backup for validation")?;

        // The real canonical database remains untouched. We intentionally run the existing
        // fail-closed restore path against an isolated Store so preview exercises the same
        // migration, integrity, semantic-load, and rollback boundary that a later authorized
        // recovery action would use.
        let store = Store::open(&workspace).context("open isolated validation store")?;
        backup::restore_user_backup(&workspace, &store, &staged_candidate)
            .context("validate managed backup through the governed restore boundary")?;
        let state = store.load().context("load validated recovery state")?;
        Ok((state.items.len(), state.active_id.is_some()))
    })();

    let _ = fs::remove_dir_all(&workspace);
    result
}

fn resolve_candidate(app_data_dir: &Path, candidate_id: &str) -> Result<PathBuf> {
    if !candidate_id.starts_with("bob-backup-")
        || !candidate_id.ends_with(".sqlite3")
        || candidate_id.contains('/')
        || candidate_id.contains('\\')
        || candidate_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
        })
    {
        bail!("invalid managed backup candidate id");
    }

    let backup_dir = app_data_dir.join(USER_BACKUP_DIR);
    let candidate = backup_dir.join(candidate_id);
    let metadata = fs::symlink_metadata(&candidate).context("inspect managed backup candidate")?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        bail!("managed backup candidate is not a regular file");
    }

    let canonical_backup_dir =
        fs::canonicalize(&backup_dir).context("resolve managed backup directory")?;
    let canonical_candidate =
        fs::canonicalize(&candidate).context("resolve managed backup candidate")?;
    if canonical_candidate.parent() != Some(canonical_backup_dir.as_path()) {
        bail!("managed backup candidate escaped the managed backup directory");
    }
    Ok(canonical_candidate)
}

fn unique_stamp() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")
        .map(|duration| duration.as_nanos())
}

#[tauri::command]
pub fn validate_recovery_backup_command(
    app: AppHandle,
    candidate_id: String,
) -> std::result::Result<RecoveryBackupPreview, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let preview = match validate_recovery_backup(&app_data_dir, &candidate_id) {
        Ok((work_item_count, has_active_item)) => RecoveryBackupPreview {
            candidate_id,
            validation: RecoveryBackupValidation::Usable,
            work_item_count: Some(work_item_count),
            has_active_item: Some(has_active_item),
            message: "This backup passed B.O.B.'s recovery checks. Nothing has been restored yet."
                .into(),
        },
        Err(_) => RecoveryBackupPreview {
            candidate_id,
            validation: RecoveryBackupValidation::Unavailable,
            work_item_count: None,
            has_active_item: None,
            message: "B.O.B. could not validate this backup for recovery. Nothing was changed."
                .into(),
        },
    };
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_managed_backup(app_data_dir: &Path, name: &str) -> Result<PathBuf> {
        let source = app_data_dir.join("source.sqlite3");
        let connection = Connection::open(&source)?;
        connection.execute_batch(
            "CREATE TABLE work_items (
                id TEXT PRIMARY KEY NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                estimate INTEGER,
                priority TEXT NOT NULL,
                due TEXT,
                status TEXT NOT NULL,
                sort_order INTEGER NOT NULL
             );
             CREATE TABLE app_state (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                active_item_id TEXT REFERENCES work_items(id) ON DELETE SET NULL,
                handoff_objective TEXT,
                handoff_state TEXT,
                handoff_next TEXT,
                larger_text INTEGER NOT NULL DEFAULT 0 CHECK (larger_text IN (0, 1)),
                reduced_motion INTEGER NOT NULL DEFAULT 0 CHECK (reduced_motion IN (0, 1))
             );
             INSERT INTO work_items
                (id, kind, title, estimate, priority, due, status, sort_order)
             VALUES ('one', 'task', 'Recover me', 15, 'high', 'Today', 'planned', 0);
             INSERT INTO app_state (singleton, active_item_id) VALUES (1, 'one');
             PRAGMA user_version = 3;",
        )?;
        drop(connection);

        let backup_dir = app_data_dir.join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let candidate = backup_dir.join(name);
        fs::copy(source, &candidate)?;
        Ok(candidate)
    }

    #[test]
    fn previews_managed_backup_without_touching_corrupt_canonical_bytes() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join("bob.sqlite3");
        fs::write(&canonical, b"corrupt canonical bytes")?;
        let original = fs::read(&canonical)?;
        create_managed_backup(directory.path(), "bob-backup-preview.sqlite3")?;

        let (count, has_active) =
            validate_recovery_backup(directory.path(), "bob-backup-preview.sqlite3")?;

        assert_eq!(count, 1);
        assert!(has_active);
        assert_eq!(fs::read(&canonical)?, original);
        Ok(())
    }

    #[test]
    fn rejects_path_traversal_candidate_ids() -> Result<()> {
        let directory = tempfile::tempdir()?;
        assert!(
            validate_recovery_backup(directory.path(), "bob-backup-../escape.sqlite3").is_err()
        );
        assert!(
            validate_recovery_backup(directory.path(), "../bob-backup-escape.sqlite3").is_err()
        );
        Ok(())
    }

    #[test]
    fn corrupt_candidate_fails_closed() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        fs::write(backup_dir.join("bob-backup-corrupt.sqlite3"), b"not sqlite")?;
        assert!(validate_recovery_backup(directory.path(), "bob-backup-corrupt.sqlite3").is_err());
        Ok(())
    }
}
