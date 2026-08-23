use serde::Serialize;
use std::{fs, io::ErrorKind, path::Path, time::UNIX_EPOCH};
use tauri::{AppHandle, State};

const USER_BACKUP_DIR: &str = "backups";
const MAX_RECOVERY_BACKUP_CANDIDATES: usize = 8;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedBackupCandidate {
    id: String,
    modified_unix_ms: Option<u64>,
    size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    mode: StartupMode,
    managed_backup_count: Option<usize>,
    managed_backup_candidates: Option<Vec<ManagedBackupCandidate>>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
enum StartupMode {
    Ready,
    RecoveryRequired,
}

#[derive(Clone, Debug)]
pub struct StartupState(StartupStatus);

impl StartupState {
    pub fn ready() -> Self {
        Self(StartupStatus {
            mode: StartupMode::Ready,
            managed_backup_count: Some(0),
            managed_backup_candidates: Some(Vec::new()),
        })
    }

    pub fn recovery_required(app_data_dir: &Path) -> Self {
        let mut managed_backup_candidates = managed_backup_candidates(app_data_dir);
        let managed_backup_count = managed_backup_candidates.as_ref().map(Vec::len);
        if let Some(candidates) = managed_backup_candidates.as_mut() {
            candidates.truncate(MAX_RECOVERY_BACKUP_CANDIDATES);
        }
        Self(StartupStatus {
            mode: StartupMode::RecoveryRequired,
            managed_backup_count,
            managed_backup_candidates,
        })
    }
}

fn managed_backup_candidates(app_data_dir: &Path) -> Option<Vec<ManagedBackupCandidate>> {
    let backup_dir = app_data_dir.join(USER_BACKUP_DIR);
    let entries = match fs::read_dir(backup_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };

    let mut candidates = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            return None;
        };
        let Ok(file_type) = entry.file_type() else {
            return None;
        };
        if !file_type.is_file() {
            continue;
        }

        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with("bob-backup-") || !name.ends_with(".sqlite3") {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            return None;
        };
        let modified_unix_ms = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .and_then(|duration| u64::try_from(duration.as_millis()).ok());

        candidates.push(ManagedBackupCandidate {
            id: name.to_owned(),
            modified_unix_ms,
            size_bytes: metadata.len(),
        });
    }

    candidates.sort_by(|left, right| {
        right
            .modified_unix_ms
            .cmp(&left.modified_unix_ms)
            .then_with(|| right.id.cmp(&left.id))
    });
    Some(candidates)
}

#[tauri::command]
pub fn startup_status(state: State<'_, StartupState>) -> StartupStatus {
    state.0.clone()
}

#[tauri::command]
pub fn restart_application(app: AppHandle) {
    // A webview reload would preserve the immutable StartupState created during Tauri setup and
    // could never recover from a transient Store-open failure. Restart the process so setup runs
    // again and canonical state is re-evaluated from disk.
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_status_lists_only_regular_managed_backup_files() -> anyhow::Result<()> {
        let directory = tempfile::tempdir()?;
        let backups = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backups)?;
        fs::write(backups.join("bob-backup-1.sqlite3"), b"one")?;
        fs::write(backups.join("bob-backup-2.sqlite3"), b"two-two")?;
        fs::write(backups.join("notes.txt"), b"ignore")?;
        fs::create_dir(backups.join("bob-backup-directory.sqlite3"))?;

        let state = StartupState::recovery_required(directory.path());
        assert_eq!(state.0.managed_backup_count, Some(2));
        let candidates = state
            .0
            .managed_backup_candidates
            .as_ref()
            .expect("known candidates");
        assert_eq!(candidates.len(), 2);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.id.starts_with("bob-backup-")));
        assert!(candidates
            .iter()
            .all(|candidate| candidate.id.ends_with(".sqlite3")));
        assert!(candidates.iter().any(|candidate| candidate.size_bytes == 3));
        assert!(candidates.iter().any(|candidate| candidate.size_bytes == 7));
        assert!(matches!(state.0.mode, StartupMode::RecoveryRequired));
        Ok(())
    }

    #[test]
    fn recovery_status_bounds_rendered_candidates_without_hiding_total_count() -> anyhow::Result<()>
    {
        let directory = tempfile::tempdir()?;
        let backups = directory.path().join(USER_BACKUP_DIR);
        fs::create_dir_all(&backups)?;
        let total = MAX_RECOVERY_BACKUP_CANDIDATES + 4;
        for index in 0..total {
            fs::write(
                backups.join(format!("bob-backup-{index}.sqlite3")),
                b"backup",
            )?;
        }

        let state = StartupState::recovery_required(directory.path());
        assert_eq!(state.0.managed_backup_count, Some(total));
        assert_eq!(
            state.0.managed_backup_candidates.as_ref().map(Vec::len),
            Some(MAX_RECOVERY_BACKUP_CANDIDATES)
        );
        Ok(())
    }

    #[test]
    fn recovery_status_reports_zero_when_backup_directory_does_not_exist() -> anyhow::Result<()> {
        let directory = tempfile::tempdir()?;
        let state = StartupState::recovery_required(directory.path());
        assert_eq!(state.0.managed_backup_count, Some(0));
        assert_eq!(
            state.0.managed_backup_candidates.as_ref().map(Vec::len),
            Some(0)
        );
        Ok(())
    }

    #[test]
    fn recovery_status_reports_unknown_when_backup_directory_cannot_be_read() -> anyhow::Result<()>
    {
        let directory = tempfile::tempdir()?;
        fs::write(directory.path().join(USER_BACKUP_DIR), b"not a directory")?;

        let state = StartupState::recovery_required(directory.path());
        assert_eq!(state.0.managed_backup_count, None);
        assert!(state.0.managed_backup_candidates.is_none());
        Ok(())
    }
}
