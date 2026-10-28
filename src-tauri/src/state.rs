use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior, MAIN_DB};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::Path, sync::Mutex, time::Duration};
use tauri::State;

const SCHEMA_VERSION: i64 = 5;
const DATABASE_NAME: &str = "bob.sqlite3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub estimate: Option<i64>,
    pub priority: String,
    pub due: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffSnapshot {
    pub objective: String,
    pub state: String,
    pub next: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkState {
    pub active_id: Option<String>,
    pub items: Vec<WorkItem>,
    pub handoff: Option<HandoffSnapshot>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityPreferences {
    pub larger_text: bool,
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatEntry {
    pub author: String,
    pub text: String,
}

pub struct Store {
    connection: Mutex<Connection>,
}

impl Store {
    pub fn open(app_data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(app_data_dir).with_context(|| {
            format!(
                "create B.O.B. app-data directory at {}",
                app_data_dir.display()
            )
        })?;

        let db_path = app_data_dir.join(DATABASE_NAME);
        let existed_before_open = db_path.exists();
        let mut connection = Connection::open(&db_path)
            .with_context(|| format!("open canonical B.O.B. database at {}", db_path.display()))?;

        connection
            .busy_timeout(Duration::from_secs(5))
            .context("configure SQLite busy timeout")?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .context("enable SQLite foreign keys")?;

        run_migrations(&mut connection, &db_path, existed_before_open)?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn load(&self) -> Result<WorkState> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        load_work_state_from_connection(&connection)
    }

    pub fn save(&self, state: &WorkState) -> Result<()> {
        validate_work_state(state)?;

        let mut connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin canonical work-state transaction")?;

        transaction
            .execute("DELETE FROM work_items", [])
            .context("replace canonical work items")?;

        for (sort_order, item) in state.items.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO work_items
                        (id, kind, title, estimate, priority, due, status, sort_order)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        item.id,
                        item.kind,
                        item.title,
                        item.estimate,
                        item.priority,
                        item.due,
                        item.status,
                        sort_order as i64,
                    ],
                )
                .with_context(|| format!("persist work item {}", item.id))?;
        }

        let (handoff_objective, handoff_state, handoff_next) = state
            .handoff
            .as_ref()
            .map(|handoff| {
                (
                    Some(handoff.objective.as_str()),
                    Some(handoff.state.as_str()),
                    Some(handoff.next.as_str()),
                )
            })
            .unwrap_or((None, None, None));

        transaction
            .execute(
                "INSERT INTO app_state
                    (singleton, active_item_id, handoff_objective, handoff_state, handoff_next)
                 VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(singleton) DO UPDATE SET
                    active_item_id = excluded.active_item_id,
                    handoff_objective = excluded.handoff_objective,
                    handoff_state = excluded.handoff_state,
                    handoff_next = excluded.handoff_next",
                params![
                    state.active_id.as_deref(),
                    handoff_objective,
                    handoff_state,
                    handoff_next
                ],
            )
            .context("persist B.O.B. application state")?;

        transaction
            .commit()
            .context("commit canonical work-state transaction")?;
        Ok(())
    }

    pub fn openrouter_settings(&self) -> Result<(Option<String>, bool, bool)> {
        let connection = self.connection.lock().map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        connection.query_row(
            "SELECT model_id, cloud_enabled, metered_enabled FROM openrouter_settings WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)? != 0, row.get::<_, i64>(2)? != 0)),
        ).context("read OpenRouter authorization")
    }

    pub fn set_openrouter_settings(&self, model: Option<&str>, cloud: bool, metered: bool) -> Result<()> {
        if let Some(model) = model {
            if model.is_empty() || model.len() > 256 || model.trim() != model
                || model.chars().any(|c| c.is_whitespace() || c.is_control()) {
                bail!("invalid OpenRouter model identifier");
            }
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        let affected = connection.execute(
            "UPDATE openrouter_settings SET model_id = ?1, cloud_enabled = ?2, metered_enabled = ?3 WHERE singleton = 1",
            params![model, i64::from(cloud), i64::from(metered)],
        )?;
        if affected != 1 {
            bail!("OpenRouter settings record missing; refusing silent authorization update");
        }
        Ok(())
    }

    pub fn load_chat(&self) -> Result<Vec<ChatEntry>> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        let mut statement =
            connection.prepare("SELECT author, text FROM chat_entries ORDER BY id ASC")?;
        let entries = statement
            .query_map([], |row| {
                Ok(ChatEntry {
                    author: row.get(0)?,
                    text: row.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    pub fn append_chat(&self, entry: ChatEntry) -> Result<Vec<ChatEntry>> {
        if !matches!(entry.author.as_str(), "user" | "bob")
            || entry.text.trim().is_empty()
            || entry.text.len() > 8_000
        {
            bail!("invalid chat entry");
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO chat_entries (author, text) VALUES (?1, ?2)",
            params![entry.author, entry.text],
        )?;
        // Retain the most recent 200 turns to bound storage and on-screen context.
        transaction.execute("DELETE FROM chat_entries WHERE id NOT IN (SELECT id FROM chat_entries ORDER BY id DESC LIMIT 200)", [])?;
        transaction.commit()?;
        drop(connection);
        self.load_chat()
    }

    pub fn clear_chat(&self) -> Result<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        connection.execute("DELETE FROM chat_entries", [])?;
        Ok(())
    }

    pub fn load_accessibility_preferences(&self) -> Result<AccessibilityPreferences> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        load_accessibility_preferences_from_connection(&connection)
    }

    pub fn save_accessibility_preferences(
        &self,
        preferences: AccessibilityPreferences,
    ) -> Result<AccessibilityPreferences> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;
        let larger_text = if preferences.larger_text { 1 } else { 0 };
        let reduced_motion = if preferences.reduced_motion { 1 } else { 0 };
        let changed = connection
            .execute(
                "UPDATE app_state
                 SET larger_text = ?1, reduced_motion = ?2
                 WHERE singleton = 1",
                params![larger_text, reduced_motion],
            )
            .context("persist accessibility preferences")?;
        if changed != 1 {
            bail!("canonical accessibility preference row is missing");
        }
        Ok(preferences)
    }

    pub(crate) fn restore_prepared_snapshot_fail_closed(
        &self,
        prepared_snapshot: &Path,
        recovery_snapshot: &Path,
    ) -> Result<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| anyhow!("canonical state lock is poisoned"))?;

        let recovery_pending = recovery_snapshot.with_file_name(".bob-pre-restore-pending.sqlite3");
        if recovery_pending.exists() {
            fs::remove_file(&recovery_pending).with_context(|| {
                format!(
                    "remove stale pending pre-restore recovery snapshot at {}",
                    recovery_pending.display()
                )
            })?;
        }

        connection
            .backup(MAIN_DB, &recovery_pending, None)
            .context("create SQLite-consistent pending pre-restore recovery snapshot")?;
        quick_check_path(&recovery_pending)
            .context("verify pending pre-restore recovery snapshot before promotion")?;

        promote_verified_recovery_snapshot(&recovery_pending, recovery_snapshot)?;

        let restore_result = restore_connection_from_snapshot(&mut connection, prepared_snapshot)
            .and_then(|_| load_work_state_from_connection(&connection).map(|_| ()))
            .and_then(|_| load_accessibility_preferences_from_connection(&connection).map(|_| ()));

        if let Err(restore_error) = restore_result {
            let rollback_result =
                restore_connection_from_snapshot(&mut connection, recovery_snapshot)
                    .and_then(|_| load_work_state_from_connection(&connection).map(|_| ()))
                    .and_then(|_| {
                        load_accessibility_preferences_from_connection(&connection).map(|_| ())
                    });

            return match rollback_result {
                Ok(()) => Err(restore_error).context(
                    "restore candidate failed after canonical mutation; canonical state was rolled back to the verified pre-restore snapshot",
                ),
                Err(rollback_error) => Err(anyhow!(
                    "restore failed: {restore_error:#}; rollback also failed: {rollback_error:#}; verified pre-restore snapshot retained at {}",
                    recovery_snapshot.display()
                )),
            };
        }

        Ok(())
    }
}

#[tauri::command]
pub fn load_chat_history(store: State<'_, Store>) -> std::result::Result<Vec<ChatEntry>, String> {
    store.load_chat().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn append_chat_message(
    store: State<'_, Store>,
    entry: ChatEntry,
) -> std::result::Result<Vec<ChatEntry>, String> {
    store.append_chat(entry).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_chat_history(store: State<'_, Store>) -> std::result::Result<(), String> {
    store.clear_chat().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_work_state(store: State<'_, Store>) -> std::result::Result<WorkState, String> {
    store.load().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_accessibility_preferences(
    store: State<'_, Store>,
) -> std::result::Result<AccessibilityPreferences, String> {
    store
        .load_accessibility_preferences()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_accessibility_preferences(
    store: State<'_, Store>,
    preferences: AccessibilityPreferences,
) -> std::result::Result<AccessibilityPreferences, String> {
    store
        .save_accessibility_preferences(preferences)
        .map_err(|error| error.to_string())
}

fn load_work_state_from_connection(connection: &Connection) -> Result<WorkState> {
    let app_state = connection
        .query_row(
            "SELECT active_item_id, handoff_objective, handoff_state, handoff_next
             FROM app_state WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()
        .context("load B.O.B. application state")?
        .unwrap_or((None, None, None, None));

    let handoff = match (app_state.1, app_state.2, app_state.3) {
        (None, None, None) => None,
        (Some(objective), Some(state), Some(next)) => Some(HandoffSnapshot {
            objective,
            state,
            next,
        }),
        _ => bail!("canonical handoff state is incomplete"),
    };

    let mut statement = connection
        .prepare(
            "SELECT id, kind, title, estimate, priority, due, status
             FROM work_items
             ORDER BY sort_order ASC, id ASC",
        )
        .context("prepare canonical work-state query")?;

    let rows = statement
        .query_map([], |row| {
            Ok(WorkItem {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                estimate: row.get(3)?,
                priority: row.get(4)?,
                due: row.get(5)?,
                status: row.get(6)?,
            })
        })
        .context("read canonical work-state rows")?;

    let items = rows
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("decode canonical work-state rows")?;
    let state = WorkState {
        active_id: app_state.0,
        items,
        handoff,
    };
    validate_work_state(&state)?;
    Ok(state)
}

fn load_accessibility_preferences_from_connection(
    connection: &Connection,
) -> Result<AccessibilityPreferences> {
    connection
        .query_row(
            "SELECT larger_text, reduced_motion FROM app_state WHERE singleton = 1",
            [],
            |row| {
                Ok(AccessibilityPreferences {
                    larger_text: row.get::<_, i64>(0)? != 0,
                    reduced_motion: row.get::<_, i64>(1)? != 0,
                })
            },
        )
        .optional()
        .context("load accessibility preferences")
        .map(|preferences| preferences.unwrap_or_default())
}

fn restore_connection_from_snapshot(connection: &mut Connection, snapshot: &Path) -> Result<()> {
    connection
        .restore(MAIN_DB, snapshot, None::<fn(rusqlite::backup::Progress)>)
        .with_context(|| format!("restore SQLite snapshot from {}", snapshot.display()))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .context("re-enable SQLite foreign keys after restore")?;
    quick_check(connection).context("verify restored SQLite integrity")?;

    let version = user_version(connection)?;
    if version != SCHEMA_VERSION {
        bail!(
            "prepared restore snapshot schema {version} does not match running schema {SCHEMA_VERSION}"
        );
    }
    Ok(())
}

fn validate_work_state(state: &WorkState) -> Result<()> {
    let mut ids = HashSet::new();

    for item in &state.items {
        if item.id.trim().is_empty() || item.id.len() > 128 {
            bail!("work item id is missing or too long");
        }
        if !ids.insert(item.id.as_str()) {
            bail!("duplicate work item id: {}", item.id);
        }
        if item.title.trim().is_empty() || item.title.len() > 500 {
            bail!("work item {} has an invalid title", item.id);
        }
        if !matches!(item.kind.as_str(), "task" | "idea" | "note" | "reminder") {
            bail!("work item {} has an invalid kind", item.id);
        }
        if !matches!(item.priority.as_str(), "low" | "normal" | "high") {
            bail!("work item {} has an invalid priority", item.id);
        }
        if !matches!(
            item.status.as_str(),
            "inbox" | "planned" | "doing" | "done" | "deferred"
        ) {
            bail!("work item {} has an invalid status", item.id);
        }
        if item
            .estimate
            .is_some_and(|estimate| estimate <= 0 || estimate > 10_080)
        {
            bail!("work item {} has an invalid estimate", item.id);
        }
        if item.due.as_ref().is_some_and(|due| due.len() > 200) {
            bail!("work item {} has an invalid due value", item.id);
        }
    }

    if let Some(active_id) = &state.active_id {
        if !ids.contains(active_id.as_str()) {
            bail!("active work item does not exist in canonical state");
        }
    }

    if let Some(handoff) = &state.handoff {
        if handoff.objective.trim().is_empty() || handoff.objective.len() > 500 {
            bail!("handoff objective is missing or too long");
        }
        if handoff.state.trim().is_empty() || handoff.state.len() > 200 {
            bail!("handoff state is missing or too long");
        }
        if handoff.next.trim().is_empty() || handoff.next.len() > 1_000 {
            bail!("handoff next action is missing or too long");
        }
    }

    Ok(())
}

fn run_migrations(
    connection: &mut Connection,
    db_path: &Path,
    existed_before_open: bool,
) -> Result<()> {
    let initial_version = user_version(connection)?;
    if initial_version > SCHEMA_VERSION {
        bail!(
            "canonical state schema {initial_version} is newer than supported schema {SCHEMA_VERSION}; refusing destructive downgrade"
        );
    }

    if initial_version == SCHEMA_VERSION {
        quick_check(connection)?;
        return Ok(());
    }

    if existed_before_open && fs::metadata(db_path).map(|meta| meta.len()).unwrap_or(0) > 0 {
        quick_check(connection)?;
        create_pre_migration_safety_copy(connection, db_path)?;
    }

    let mut version = initial_version;
    while version < SCHEMA_VERSION {
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin schema migration transaction")?;

        match version {
            0 => transaction
                .execute_batch(
                    "CREATE TABLE IF NOT EXISTS work_items (
                        id TEXT PRIMARY KEY NOT NULL,
                        kind TEXT NOT NULL,
                        title TEXT NOT NULL,
                        estimate INTEGER,
                        priority TEXT NOT NULL,
                        due TEXT,
                        status TEXT NOT NULL,
                        sort_order INTEGER NOT NULL
                    );

                    CREATE TABLE IF NOT EXISTS app_state (
                        singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                        active_item_id TEXT REFERENCES work_items(id) ON DELETE SET NULL
                    );

                    INSERT OR IGNORE INTO app_state (singleton, active_item_id) VALUES (1, NULL);
                    PRAGMA user_version = 1;",
                )
                .context("apply schema migration 1")?,
            1 => transaction
                .execute_batch(
                    "ALTER TABLE app_state ADD COLUMN handoff_objective TEXT;
                     ALTER TABLE app_state ADD COLUMN handoff_state TEXT;
                     ALTER TABLE app_state ADD COLUMN handoff_next TEXT;
                     PRAGMA user_version = 2;",
                )
                .context("apply schema migration 2")?,
            2 => transaction
                .execute_batch(
                    "ALTER TABLE app_state ADD COLUMN larger_text INTEGER NOT NULL DEFAULT 0
                        CHECK (larger_text IN (0, 1));
                     ALTER TABLE app_state ADD COLUMN reduced_motion INTEGER NOT NULL DEFAULT 0
                        CHECK (reduced_motion IN (0, 1));
                     PRAGMA user_version = 3;",
                )
                .context("apply schema migration 3")?,
            3 => transaction
                .execute_batch(
                    "CREATE TABLE chat_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    author TEXT NOT NULL CHECK(author IN ('user', 'bob')),
                    text TEXT NOT NULL CHECK(length(text) > 0)
                );
                PRAGMA user_version = 4;",
                )
                .context("apply schema migration 4")?,
            4 => transaction.execute_batch(
                "CREATE TABLE openrouter_settings (
                    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
                    model_id TEXT,
                    cloud_enabled INTEGER NOT NULL DEFAULT 0 CHECK(cloud_enabled IN (0, 1)),
                    metered_enabled INTEGER NOT NULL DEFAULT 0 CHECK(metered_enabled IN (0, 1))
                );
                INSERT INTO openrouter_settings(singleton, model_id, cloud_enabled, metered_enabled)
                VALUES (1, NULL, 0, 0);
                PRAGMA user_version = 5;",
            ).context("apply schema migration 5")?,
            other => bail!("no migration path exists from schema version {other}"),
        }

        transaction.commit().context("commit schema migration")?;
        let next_version = user_version(connection)?;
        if next_version <= version {
            bail!("schema migration did not advance from version {version}");
        }
        version = next_version;
    }

    if version != SCHEMA_VERSION {
        bail!("schema migration ended at version {version}, expected {SCHEMA_VERSION}");
    }
    quick_check(connection)?;
    Ok(())
}

fn user_version(connection: &Connection) -> Result<i64> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .context("read SQLite schema version")
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
        .with_context(|| format!("open recovery snapshot read-only at {}", path.display()))?;
    quick_check(&connection)
}

fn promote_verified_recovery_snapshot(pending: &Path, target: &Path) -> Result<()> {
    fs::rename(pending, target).with_context(|| {
        format!(
            "promote verified pre-restore recovery snapshot from {} to {} while preserving the prior target if replacement fails",
            pending.display(),
            target.display()
        )
    })
}

fn create_pre_migration_safety_copy(connection: &Connection, db_path: &Path) -> Result<()> {
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow!("canonical database path has no parent directory"))?;
    let recovery_dir = parent.join("recovery");
    fs::create_dir_all(&recovery_dir).context("create recovery directory")?;

    let pending = recovery_dir.join("bob-pre-migration-pending.sqlite3");
    let newest = recovery_dir.join("bob-pre-migration-1.sqlite3");
    let older = recovery_dir.join("bob-pre-migration-2.sqlite3");

    if pending.exists() {
        fs::remove_file(&pending).context("remove stale pending recovery snapshot")?;
    }

    let target = path_for_sqlite(&pending)?;
    connection
        .execute("VACUUM INTO ?1", params![target])
        .context("create SQLite-consistent pending pre-migration safety copy")?;
    quick_check_path(&pending).context("verify pending pre-migration safety copy")?;

    if older.exists() {
        fs::remove_file(&older).context("remove oldest pre-migration safety copy")?;
    }
    if newest.exists() {
        fs::rename(&newest, &older).context("rotate newest pre-migration safety copy")?;
    }
    fs::rename(&pending, &newest).context("promote verified pre-migration safety copy")?;
    Ok(())
}

fn path_for_sqlite(path: &Path) -> Result<String> {
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("SQLite backup path is not valid Unicode"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> WorkState {
        WorkState {
            active_id: Some("one".into()),
            items: vec![WorkItem {
                id: "one".into(),
                kind: "task".into(),
                title: "First durable task".into(),
                estimate: Some(15),
                priority: "high".into(),
                due: Some("Today".into()),
                status: "planned".into(),
            }],
            handoff: Some(HandoffSnapshot {
                objective: "First durable task".into(),
                state: "In progress".into(),
                next: "Reopen the context and make the first concrete change.".into(),
            }),
        }
    }

    #[test]
    fn round_trips_work_state_and_handoff() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        let expected = sample_state();

        store.save(&expected)?;

        assert_eq!(store.load()?, expected);
        Ok(())
    }

    #[test]
    fn chat_survives_store_reopen_and_rejects_invalid_entries() -> Result<()> {
        let directory = tempfile::tempdir()?;
        {
            let store = Store::open(directory.path())?;
            store.append_chat(ChatEntry {
                author: "user".into(),
                text: "Return to this tomorrow".into(),
            })?;
            store.append_chat(ChatEntry {
                author: "bob".into(),
                text: "I'll keep the context.".into(),
            })?;
            assert!(store
                .append_chat(ChatEntry {
                    author: "system".into(),
                    text: "bad".into()
                })
                .is_err());
        }
        let reopened = Store::open(directory.path())?;
        let history = reopened.load_chat()?;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].text, "Return to this tomorrow");
        assert_eq!(history[1].author, "bob");
        Ok(())
    }

    #[test]
    fn rejects_chat_over_byte_limit_without_partial_write() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        store.append_chat(ChatEntry {
            author: "user".into(),
            text: "valid".into(),
        })?;
        assert!(store
            .append_chat(ChatEntry {
                author: "user".into(),
                text: "🧠".repeat(2_001),
            })
            .is_err());
        assert_eq!(store.load_chat()?.len(), 1);
        assert_eq!(store.load_chat()?[0].text, "valid");
        Ok(())
    }

    #[test]
    fn missing_openrouter_settings_row_rejects_authorization_update() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        {
            let connection = store.connection.lock().unwrap();
            connection.execute("DELETE FROM openrouter_settings WHERE singleton = 1", [])?;
        }
        assert!(store
            .set_openrouter_settings(Some("vendor/model"), true, true)
            .is_err());
        assert!(store.openrouter_settings().is_err());
        Ok(())
    }

    #[test]
    fn openrouter_settings_are_deny_by_default_and_survive_reopen() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        assert_eq!(store.openrouter_settings()?, (None, false, false));
        assert!(store.set_openrouter_settings(Some("bad model"), true, true).is_err());
        assert_eq!(store.openrouter_settings()?, (None, false, false));
        store.set_openrouter_settings(Some("vendor/approved-model"), true, false)?;
        drop(store);
        let reopened = Store::open(directory.path())?;
        assert_eq!(
            reopened.openrouter_settings()?,
            (Some("vendor/approved-model".into()), true, false)
        );
        reopened.set_openrouter_settings(None, false, false)?;
        assert_eq!(reopened.openrouter_settings()?, (None, false, false));
        Ok(())
    }

    #[test]
    fn clearing_chat_preserves_canonical_work_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        store.save(&sample_state())?;
        store.append_chat(ChatEntry {
            author: "user".into(),
            text: "Private context".into(),
        })?;
        store.clear_chat()?;
        assert!(store.load_chat()?.is_empty());
        assert_eq!(store.load()?, sample_state());
        drop(store);
        assert!(Store::open(directory.path())?.load_chat()?.is_empty());
        Ok(())
    }

    #[test]
    fn chat_history_retains_latest_two_hundred_messages() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;

        for number in 0..205 {
            store.append_chat(ChatEntry {
                author: "user".into(),
                text: format!("Message {number}"),
            })?;
        }

        let history = store.load_chat()?;
        assert_eq!(history.len(), 200);
        assert_eq!(history[0].text, "Message 5");
        assert_eq!(history[199].text, "Message 204");
        Ok(())
    }

    #[test]
    fn v3_to_v4_migration_preserves_existing_work_and_preferences() -> Result<()> {
        let directory = tempfile::tempdir()?;
        {
            let store = Store::open(directory.path())?;
            store.save(&sample_state())?;
            store.save_accessibility_preferences(AccessibilityPreferences {
                larger_text: true,
                reduced_motion: false,
            })?;
        }

        let db_path = directory.path().join(DATABASE_NAME);
        let legacy = Connection::open(&db_path)?;
        legacy.execute_batch("DROP TABLE chat_entries; PRAGMA user_version = 3;")?;
        drop(legacy);

        let upgraded = Store::open(directory.path())?;
        assert_eq!(upgraded.load()?, sample_state());
        assert_eq!(
            upgraded.load_accessibility_preferences()?,
            AccessibilityPreferences {
                larger_text: true,
                reduced_motion: false,
            }
        );
        assert!(upgraded.load_chat()?.is_empty());
        assert!(directory
            .path()
            .join("recovery")
            .join("bob-pre-migration-1.sqlite3")
            .exists());
        Ok(())
    }

    #[test]
    fn round_trips_accessibility_preferences() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        let expected = AccessibilityPreferences {
            larger_text: true,
            reduced_motion: true,
        };

        assert_eq!(store.save_accessibility_preferences(expected)?, expected);
        assert_eq!(store.load_accessibility_preferences()?, expected);
        Ok(())
    }

    #[test]
    fn rejects_unknown_active_item_without_mutating_state() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        let mut invalid = sample_state();
        invalid.active_id = Some("missing".into());

        assert!(store.save(&invalid).is_err());
        assert!(store.load()?.items.is_empty());
        Ok(())
    }

    #[test]
    fn failed_recovery_promotion_preserves_existing_snapshot() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let pending = directory.path().join("missing-pending.sqlite3");
        let target = directory.path().join("bob-pre-restore-last.sqlite3");
        fs::write(&target, b"known-good")?;

        assert!(promote_verified_recovery_snapshot(&pending, &target).is_err());
        assert_eq!(fs::read(&target)?, b"known-good");
        Ok(())
    }

    #[test]
    fn recovery_promotion_replaces_existing_snapshot_without_delete_window() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let pending = directory.path().join("pending.sqlite3");
        let target = directory.path().join("bob-pre-restore-last.sqlite3");
        fs::write(&pending, b"new-known-good")?;
        fs::write(&target, b"old-known-good")?;

        promote_verified_recovery_snapshot(&pending, &target)?;

        assert!(!pending.exists());
        assert_eq!(fs::read(&target)?, b"new-known-good");
        Ok(())
    }

    #[test]
    fn migrates_v1_state_to_v3_without_losing_work() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let db_path = directory.path().join(DATABASE_NAME);
        let legacy = Connection::open(&db_path)?;
        legacy.execute_batch(
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
             VALUES ('one', 'task', 'Preserve me', 15, 'high', 'Today', 'planned', 0);
             INSERT INTO app_state (singleton, active_item_id) VALUES (1, 'one');
             PRAGMA user_version = 1;",
        )?;
        drop(legacy);

        let store = Store::open(directory.path())?;
        let migrated = store.load()?;

        assert_eq!(migrated.active_id.as_deref(), Some("one"));
        assert_eq!(migrated.items.len(), 1);
        assert_eq!(migrated.items[0].title, "Preserve me");
        assert_eq!(migrated.handoff, None);
        assert_eq!(
            store.load_accessibility_preferences()?,
            AccessibilityPreferences::default()
        );
        Ok(())
    }

    #[test]
    fn creates_consistent_safety_copy_before_migrating_existing_database() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let db_path = directory.path().join(DATABASE_NAME);
        let legacy = Connection::open(&db_path)?;
        legacy.execute("CREATE TABLE legacy_marker (value TEXT NOT NULL)", [])?;
        legacy.execute("INSERT INTO legacy_marker (value) VALUES ('keep-me')", [])?;
        drop(legacy);

        let _store = Store::open(directory.path())?;

        let safety_copy = directory
            .path()
            .join("recovery")
            .join("bob-pre-migration-1.sqlite3");
        assert!(safety_copy.exists());
        let copy = Connection::open(safety_copy)?;
        let marker: String =
            copy.query_row("SELECT value FROM legacy_marker", [], |row| row.get(0))?;
        assert_eq!(marker, "keep-me");
        Ok(())
    }
}
