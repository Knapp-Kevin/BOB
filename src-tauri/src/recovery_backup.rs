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
const VALIDATION_WORKSPACE_PREFIX: &str = "candidate-";

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

pub fn cleanup_stale_validation_workspaces(app_data_dir: &Path) -> Result<()> {
    let validation_root = app_data_dir.join(VALIDATION_ROOT);
    if !validation_root.exists() {
        return Ok(());
    }

    let root_metadata = fs::symlink_metadata(&validation_root)
        .context("inspect recovery validation root before cleanup")?;
    if root_metadata.file_type().is_symlink() || !root_metadata.file_type().is_dir() {
        bail!("recovery validation root is not a regular directory");
    }

    for entry in fs::read_dir(&validation_root).context("read recovery validation root")? {
        let entry = entry.context("read recovery validation workspace entry")?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(VALIDATION_WORKSPACE_PREFIX) {
            continue;
        }

        let metadata = fs::symlink_metadata(entry.path())
            .context("inspect stale recovery validation workspace")?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            bail!("stale recovery validation workspace is not a regular directory");
        }
        fs::remove_dir_all(entry.path()).context("remove stale recovery validation workspace")?;
    }

    Ok(())
}

pub fn validate_recovery_backup(app_data_dir: &Path, candidate_id: &str) -> Result<(usize, bool)> {
    let candidate = resolve_candidate(app_data_dir, candidate_id)?;
    let validation_root = app_data_dir.join(VALIDATION_ROOT);
    fs::create_dir_all(&validation_root).context("create recovery validation root")?;

    // Stale workspace cleanup belongs to application startup, before any validation command can
    // be active. Do not reap candidate-* directories here: recovery previews may overlap, and one
    // validation must never delete another validation's live isolated workspace.
    let workspace =
        validation_root.join(format!("{VALIDATION_WORKSPACE_PREFIX}{}", unique_stamp()?));
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

    let cleanup_result =
        fs::remove_dir_all(&workspace).context("remove recovery validation workspace");
    match (result, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(validation_error), Ok(())) => Err(validation_error),
        (Err(validation_error), Err(cleanup_error)) => Err(validation_error.context(format!(
            "recovery validation workspace cleanup also failed: {cleanup_error:#}"
        ))),
    }
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

    fn assert_no_validation_workspaces(app_data_dir: &Path) -> Result<()> {
        let validation_root = app_data_dir.join(VALIDATION_ROOT);
        if !validation_root.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(validation_root)? {
            let entry = entry?;
            let name = entry.file_name();
            assert!(
                !name
                    .to_string_lossy()
                    .starts_with(VALIDATION_WORKSPACE_PREFIX),
                "validation workspace should have been removed"
            );
        }
        Ok(())
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
        assert_no_validation_workspaces(directory.path())?;
        Ok(())
    }

    #[test]
    fn validation_does_not_remove_another_active_candidate_workspace() -> Result<()> {
        let directory = tempfile::tempdir()?;
        create_managed_backup(directory.path(), "bob-backup-preview.sqlite3")?;

        let validation_root = directory.path().join(VALIDATION_ROOT);
        let active_workspace = validation_root.join("candidate-active");
        fs::create_dir_all(&active_workspace)?;
        let sentinel = active_workspace.join("in-use");
        fs::write(&sentinel, b"another preview owns this workspace")?;

        validate_recovery_backup(directory.path(), "bob-backup-preview.sqlite3")?;

        assert!(active_workspace.exists());
        assert_eq!(fs::read(&sentinel)?, b"another preview owns this workspace");
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
    fn corrupt_candidate_fails_closed_and_removes_workspace() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        fs::write(backup_dir.join("bob-backup-corrupt.sqlite3"), b"not sqlite")?;
        assert!(validate_recovery_backup(directory.path(), "bob-backup-corrupt.sqlite3").is_err());
        assert_no_validation_workspaces(directory.path())?;
        Ok(())
    }

    #[test]
    fn removes_stale_candidate_workspaces_without_touching_other_entries() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let validation_root = directory.path().join(VALIDATION_ROOT);
        let stale = validation_root.join("candidate-stale");
        let unrelated = validation_root.join("keep-me");
        fs::create_dir_all(&stale)?;
        fs::create_dir_all(&unrelated)?;
        fs::write(stale.join("bob.sqlite3"), b"stale duplicate state")?;

        cleanup_stale_validation_workspaces(directory.path())?;

        assert!(!stale.exists());
        assert!(unrelated.exists());
        Ok(())
    }
}
