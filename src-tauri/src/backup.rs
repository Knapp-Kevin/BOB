use crate::state::Store;
use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{Connection, OpenFlags, MAIN_DB};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, State};

const DATABASE_NAME: &str = "bob.sqlite3";
const USER_BACKUP_DIR: &str = "backups";
const RESTORE_VALIDATION_DIR: &str = "restore-validation";
const RECOVERY_DIR: &str = "recovery";
const PRE_RESTORE_RECOVERY_NAME: &str = "bob-pre-restore-last.sqlite3";

struct StagedRestore {
    workspace: PathBuf,
    database: PathBuf,
}

pub fn create_user_backup(app_data_dir: &Path) -> Result<PathBuf> {
    let source_path = app_data_dir.join(DATABASE_NAME);
    if !source_path.is_file() {
        bail!(
            "canonical B.O.B. database does not exist at {}",
            source_path.display()
        );
    }

    let backup_dir = app_data_dir.join(USER_BACKUP_DIR);
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("create user-backup directory at {}", backup_dir.display()))?;

    let stamp = unique_stamp()?;
    let pending = backup_dir.join(format!(".bob-backup-{stamp}.pending.sqlite3"));
    let completed = backup_dir.join(format!("bob-backup-{stamp}.sqlite3"));

    let result = (|| {
        let source = Connection::open_with_flags(&source_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| {
                format!(
                    "open canonical B.O.B. database read-only at {}",
                    source_path.display()
                )
            })?;
        source
            .backup(MAIN_DB, &pending, None)
            .context("create SQLite-consistent user backup")?;
        quick_check_path(&pending).context("verify completed user backup")?;
        fs::rename(&pending, &completed).context("promote verified user backup")?;
        Ok(completed.clone())
    })();

    if result.is_err() && pending.exists() {
        let _ = fs::remove_file(&pending);
    }

    result
}

/// Validate a candidate user backup without mutating canonical state.
///
/// This has no non-test caller yet: `restore_user_backup` stages the candidate itself, and this
/// slice deliberately does not add a separate validate command to the IPC surface (#69 keeps the
/// interface narrow). It is scoped to tests until the restore UX slice needs a preview step, so
/// that `cargo clippy --all-targets -- -D warnings` stays green without suppressing dead-code
/// detection for the module.
#[cfg(test)]
fn validate_user_backup_candidate(app_data_dir: &Path, candidate: &Path) -> Result<()> {
    let staged = stage_user_backup_candidate(app_data_dir, candidate)?;
    cleanup_staged_restore(staged)
}

/// Restore one of B.O.B.'s managed SQLite backups through the serialized Store boundary.
///
/// The candidate is first staged and migrated in isolation. Canonical mutation occurs only after
/// that succeeds. `Store` creates and verifies a single bounded pre-restore recovery snapshot,
/// restores the prepared candidate into the live connection, validates canonical state while the
/// Store lock is still held, and automatically rolls back on any failure.
pub fn restore_user_backup(app_data_dir: &Path, store: &Store, candidate: &Path) -> Result<()> {
    let managed_candidate = resolve_managed_backup_candidate(app_data_dir, candidate)?;
    let staged = stage_user_backup_candidate(app_data_dir, &managed_candidate)?;

    let recovery_dir = app_data_dir.join(RECOVERY_DIR);
    fs::create_dir_all(&recovery_dir).with_context(|| {
        format!(
            "create pre-restore recovery directory at {}",
            recovery_dir.display()
        )
    })?;
    let recovery_snapshot = recovery_dir.join(PRE_RESTORE_RECOVERY_NAME);

    let result = store.restore_prepared_snapshot_fail_closed(&staged.database, &recovery_snapshot);

    // Staging is disposable. A cleanup failure must not misreport a completed canonical restore as
    // failed. The bounded pre-restore snapshot is intentionally retained as the latest known-good
    // state before restore and is replaced on the next restore attempt.
    let _ = cleanup_staged_restore(staged);
    result
}

fn stage_user_backup_candidate(app_data_dir: &Path, candidate: &Path) -> Result<StagedRestore> {
    if !candidate.is_file() {
        bail!(
            "restore candidate does not exist at {}",
            candidate.display()
        );
    }

    let source = Connection::open_with_flags(candidate, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| {
            format!(
                "open restore candidate read-only at {}",
                candidate.display()
            )
        })?;
    quick_check(&source).context("verify restore-candidate SQLite integrity")?;

    if !table_exists(&source, "work_items")? || !table_exists(&source, "app_state")? {
        bail!("restore candidate is not a recognized B.O.B. canonical-state backup");
    }
    require_app_state_singleton(&source)?;

    let validation_root = app_data_dir.join(RESTORE_VALIDATION_DIR);
    fs::create_dir_all(&validation_root).with_context(|| {
        format!(
            "create restore-validation directory at {}",
            validation_root.display()
        )
    })?;

    let validation_dir = validation_root.join(format!("candidate-{}", unique_stamp()?));
    fs::create_dir(&validation_dir).context("create isolated restore-validation workspace")?;
    let staged_database = validation_dir.join(DATABASE_NAME);

    let result = (|| {
        source
            .backup(MAIN_DB, &staged_database, None)
            .context("stage restore candidate through SQLite backup API")?;
        quick_check_path(&staged_database).context("verify staged restore candidate")?;

        let store = Store::open(&validation_dir)
            .context("validate restore candidate schema and migrations")?;
        store
            .load()
            .context("validate restored canonical work state")?;
        store
            .load_accessibility_preferences()
            .context("validate restored accessibility preferences")?;
        Ok(StagedRestore {
            workspace: validation_dir.clone(),
            database: staged_database.clone(),
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&validation_dir);
    }
    result
}

fn cleanup_staged_restore(staged: StagedRestore) -> Result<()> {
    fs::remove_dir_all(&staged.workspace).with_context(|| {
        format!(
            "remove restore-validation workspace at {}",
            staged.workspace.display()
        )
    })
}

fn resolve_managed_backup_candidate(app_data_dir: &Path, candidate: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(candidate)
        .with_context(|| format!("inspect restore candidate at {}", candidate.display()))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        bail!("restore candidate must be a regular B.O.B. backup file");
    }

    let backup_dir = app_data_dir.join(USER_BACKUP_DIR);
    let canonical_backup_dir = fs::canonicalize(&backup_dir).with_context(|| {
        format!(
            "resolve B.O.B. user-backup directory at {}",
            backup_dir.display()
        )
    })?;
    let canonical_candidate = fs::canonicalize(candidate)
        .with_context(|| format!("resolve restore candidate at {}", candidate.display()))?;

    if canonical_candidate.parent() != Some(canonical_backup_dir.as_path()) {
        bail!("restore candidate must be selected from B.O.B.'s managed backup directory");
    }

    let name = canonical_candidate
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("restore candidate filename is not valid Unicode"))?;
    if !name.starts_with("bob-backup-") || !name.ends_with(".sqlite3") {
        bail!("restore candidate is not a managed B.O.B. user backup");
    }

    Ok(canonical_candidate)
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )
        .context("inspect restore-candidate schema")?;
    Ok(count == 1)
}

fn require_app_state_singleton(connection: &Connection) -> Result<()> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM app_state WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .context("inspect required restore-candidate app_state singleton")?;
    if count != 1 {
        bail!("restore candidate is missing required B.O.B. app_state singleton row");
    }
    Ok(())
}

fn unique_stamp() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")
        .map(|duration| duration.as_nanos())
}

fn quick_check(connection: &Connection) -> Result<()> {
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .context("run SQLite quick_check")?;
    if result != "ok" {
        bail!("SQLite integrity check failed: {result}");
    }
    Ok(())
}

fn quick_check_path(path: &Path) -> Result<()> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open SQLite snapshot read-only at {}", path.display()))?;
    quick_check(&connection)
}

#[tauri::command]
pub fn create_user_backup_command(app: AppHandle) -> std::result::Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    create_user_backup(&app_data_dir)
        .and_then(|path| {
            path.to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| anyhow!("user backup path is not valid Unicode"))
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn restore_user_backup_command(
    app: AppHandle,
    store: State<'_, Store>,
    backup_path: String,
) -> std::result::Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    restore_user_backup(&app_data_dir, store.inner(), Path::new(&backup_path))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AccessibilityPreferences, ChatEntry, HandoffSnapshot, WorkItem, WorkState};

    fn create_bob_v3_database(path: &Path) -> Result<()> {
        let connection = Connection::open(path)?;
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
             VALUES ('one', 'task', 'Preserved task', 15, 'high', 'Today', 'planned', 0);
             INSERT INTO app_state
                (singleton, active_item_id, handoff_objective, handoff_state, handoff_next, larger_text, reduced_motion)
             VALUES (1, 'one', 'Preserved objective', 'In progress', 'Continue safely', 1, 1);
             PRAGMA user_version = 3;",
        )?;
        Ok(())
    }

    fn create_bob_v1_database(path: &Path) -> Result<()> {
        let connection = Connection::open(path)?;
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
                active_item_id TEXT REFERENCES work_items(id) ON DELETE SET NULL
             );
             INSERT INTO work_items
                (id, kind, title, estimate, priority, due, status, sort_order)
             VALUES ('legacy', 'task', 'Legacy restored task', 20, 'normal', NULL, 'planned', 0);
             INSERT INTO app_state (singleton, active_item_id) VALUES (1, 'legacy');
             PRAGMA user_version = 1;",
        )?;
        Ok(())
    }

    #[test]
    fn creates_and_validates_bob_user_backup() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let source_path = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&source_path)?;

        let backup_path = create_user_backup(directory.path())?;
        let expected_dir = directory.path().join(USER_BACKUP_DIR);

        assert!(backup_path.is_file());
        assert_eq!(backup_path.parent(), Some(expected_dir.as_path()));
        assert!(!backup_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .contains("pending"));
        validate_user_backup_candidate(directory.path(), &backup_path)?;
        Ok(())
    }

    #[test]
    fn restores_managed_backup_round_trip() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        let expected_state = store.load()?;
        let expected_preferences = store.load_accessibility_preferences()?;
        let backup_path = create_user_backup(directory.path())?;

        store.save(&WorkState {
            active_id: None,
            items: vec![WorkItem {
                id: "changed".into(),
                kind: "task".into(),
                title: "Changed after backup".into(),
                estimate: Some(30),
                priority: "low".into(),
                due: None,
                status: "inbox".into(),
            }],
            handoff: Some(HandoffSnapshot {
                objective: "Changed objective".into(),
                state: "Changed".into(),
                next: "Undo this by restoring".into(),
            }),
        })?;
        store.save_accessibility_preferences(AccessibilityPreferences::default())?;

        restore_user_backup(directory.path(), &store, &backup_path)?;

        assert_eq!(store.load()?, expected_state);
        assert_eq!(
            store.load_accessibility_preferences()?,
            expected_preferences
        );
        assert!(directory
            .path()
            .join(RECOVERY_DIR)
            .join(PRE_RESTORE_RECOVERY_NAME)
            .is_file());
        Ok(())
    }

    #[test]
    fn backup_and_restore_preserve_chat_history_with_work() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        store.append_chat(ChatEntry {
            author: "user".into(),
            text: "Remember the important context".into(),
        })?;
        let expected_state = store.load()?;
        let expected_chat = store.load_chat()?;
        let backup = create_user_backup(directory.path())?;

        store.append_chat(ChatEntry {
            author: "bob".into(),
            text: "A later response".into(),
        })?;
        store.clear_chat()?;
        store.save(&WorkState {
            active_id: None,
            items: vec![],
            handoff: None,
        })?;

        restore_user_backup(directory.path(), &store, &backup)?;

        assert_eq!(store.load()?, expected_state);
        assert_eq!(store.load_chat()?, expected_chat);
        drop(store);
        let reopened = Store::open(directory.path())?;
        assert_eq!(reopened.load_chat()?, expected_chat);
        Ok(())
    }

    #[test]
    fn clearing_chat_then_restoring_older_backup_keeps_empty_history() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        store.append_chat(ChatEntry {
            author: "user".into(),
            text: "Not present in older snapshot".into(),
        })?;

        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let older = backup_dir.join("bob-backup-older-v3.sqlite3");
        create_bob_v3_database(&older)?;

        store.clear_chat()?;
        restore_user_backup(directory.path(), &store, &older)?;

        assert!(store.load_chat()?.is_empty());
        assert_eq!(store.load()?.items[0].title, "Preserved task");
        Ok(())
    }

    #[test]
    fn restore_migrates_valid_older_schema_forward() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;

        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let legacy_backup = backup_dir.join("bob-backup-legacy.sqlite3");
        create_bob_v1_database(&legacy_backup)?;

        restore_user_backup(directory.path(), &store, &legacy_backup)?;

        let restored = store.load()?;
        assert_eq!(restored.active_id.as_deref(), Some("legacy"));
        assert_eq!(restored.items.len(), 1);
        assert_eq!(restored.items[0].title, "Legacy restored task");
        assert_eq!(restored.handoff, None);
        assert_eq!(
            store.load_accessibility_preferences()?,
            AccessibilityPreferences::default()
        );
        Ok(())
    }

    #[test]
    fn rejects_non_bob_sqlite_without_mutating_canonical_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;

        let candidate = directory.path().join("foreign.sqlite3");
        let foreign = Connection::open(&candidate)?;
        foreign.execute("CREATE TABLE unrelated (value TEXT NOT NULL)", [])?;
        drop(foreign);

        assert!(validate_user_backup_candidate(directory.path(), &candidate).is_err());

        let store = Store::open(directory.path())?;
        let state = store.load()?;
        assert_eq!(state.active_id.as_deref(), Some("one"));
        assert_eq!(state.items[0].title, "Preserved task");
        Ok(())
    }

    #[test]
    fn rejects_corrupt_non_sqlite_candidate_without_mutating_canonical_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        let expected = store.load()?;

        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let candidate = backup_dir.join("bob-backup-corrupt.sqlite3");
        fs::write(&candidate, b"this is not a sqlite database")?;

        assert!(restore_user_backup(directory.path(), &store, &candidate).is_err());
        assert_eq!(store.load()?, expected);
        Ok(())
    }

    #[test]
    fn rejects_missing_app_state_singleton_without_mutating_canonical_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        let expected_state = store.load()?;
        let expected_preferences = store.load_accessibility_preferences()?;

        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let candidate = backup_dir.join("bob-backup-missing-app-state.sqlite3");
        create_bob_v3_database(&candidate)?;
        let malformed = Connection::open(&candidate)?;
        malformed.execute("DELETE FROM app_state WHERE singleton = 1", [])?;
        drop(malformed);

        assert!(restore_user_backup(directory.path(), &store, &candidate).is_err());
        assert_eq!(store.load()?, expected_state);
        assert_eq!(
            store.load_accessibility_preferences()?,
            expected_preferences
        );
        Ok(())
    }

    #[test]
    fn rejects_newer_schema_without_mutating_canonical_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        let expected = store.load()?;

        let backup_dir = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backup_dir)?;
        let candidate = backup_dir.join("bob-backup-future.sqlite3");
        create_bob_v3_database(&candidate)?;
        let future = Connection::open(&candidate)?;
        future.pragma_update(None, "user_version", 999)?;
        drop(future);

        assert!(restore_user_backup(directory.path(), &store, &candidate).is_err());
        assert_eq!(store.load()?, expected);
        Ok(())
    }

    #[test]
    fn semantic_restore_failure_rolls_back_to_verified_pre_restore_snapshot() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let store = Store::open(directory.path())?;
        let expected = store.load()?;

        let invalid = directory.path().join("invalid-prepared.sqlite3");
        create_bob_v3_database(&invalid)?;
        let invalid_connection = Connection::open(&invalid)?;
        invalid_connection.execute(
            "UPDATE work_items SET priority = 'urgent' WHERE id = 'one'",
            [],
        )?;
        drop(invalid_connection);

        let recovery_dir = directory.path().join(RECOVERY_DIR);
        fs::create_dir_all(&recovery_dir)?;
        let recovery = recovery_dir.join("rollback-test.sqlite3");

        assert!(store
            .restore_prepared_snapshot_fail_closed(&invalid, &recovery)
            .is_err());
        assert_eq!(store.load()?, expected);
        quick_check_path(&recovery)?;
        Ok(())
    }

    #[test]
    fn managed_backup_schema_contains_no_credential_storage() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let canonical = directory.path().join(DATABASE_NAME);
        create_bob_v3_database(&canonical)?;
        let backup_path = create_user_backup(directory.path())?;
        let backup = Connection::open_with_flags(&backup_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut tables = backup
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name ASC")?;
        let table_names = tables
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        assert_eq!(table_names, vec!["app_state", "work_items"]);
        let schema: String = backup.query_row(
            "SELECT group_concat(sql, ' ') FROM sqlite_master WHERE type = 'table'",
            [],
            |row| row.get(0),
        )?;
        let normalized = schema.to_ascii_lowercase();
        assert!(!normalized.contains("credential"));
        assert!(!normalized.contains("secret"));
        assert!(!normalized.contains("api_key"));
        assert!(!normalized.contains("gemini"));
        Ok(())
    }
}
