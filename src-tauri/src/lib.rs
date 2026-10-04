mod agent;
mod backup;
mod export;
mod gemini;
pub mod ollama;
mod planner;
mod proposals;
mod recovery_backup;
pub mod runtime;
mod secrets;
mod startup;
mod state;
mod work;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;

            // Do not globally reap recovery-validation/candidate-* workspaces at startup. Another
            // B.O.B. process may still own one, and without a real cross-process ownership/liveness
            // boundary we cannot safely distinguish stale duplicate state from an active preview.
            // Each validation remains responsible for removing its own workspace before reporting
            // success; interrupted-process artifacts are therefore preserved rather than deleted
            // on an unproven ownership assumption.

            match state::Store::open(&data_dir).and_then(|store| {
                work::normalize_store(&store)?;
                Ok(store)
            }) {
                Ok(store) => {
                    app.manage(store);
                    app.manage(startup::StartupState::ready());

                    // Inference is optional. A provider credential-store initialization problem must
                    // not make deterministic B.O.B. fail to launch.
                    if let Ok(gemini) = gemini::GeminiCredentials::new() {
                        app.manage(gemini);
                    }
                }
                Err(_) => {
                    // Canonical state remains untouched. The frontend reads this restricted startup
                    // status before invoking any command that requires Store-managed state.
                    app.manage(startup::StartupState::recovery_required(&data_dir));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            startup::startup_status,
            startup::restart_application,
            recovery_backup::validate_recovery_backup_command,
            state::load_work_state,
            state::load_accessibility_preferences,
            state::set_accessibility_preferences,
            planner::plan_remaining_work,
            planner::replan_remaining_work,
            work::capture_item,
            work::classify_inbox_item,
            work::start_current_work,
            work::defer_current_work,
            work::toggle_task_completed,
            work::select_next_task,
            work::save_current_handoff,
            work::clear_handoff,
            proposals::apply_next_action_proposal,
            agent::bob_assist,
            backup::create_user_backup_command,
            backup::restore_user_backup_command,
            export::export_portable_state,
            gemini::gemini_credential_status,
            gemini::configure_gemini_credential,
            gemini::remove_gemini_credential,
            gemini::generate_gemini_context
        ])
        .run(tauri::generate_context!())
        .expect("error while running B.O.B.");
}
