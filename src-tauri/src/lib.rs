mod agent;
mod backup;
mod export;
mod gemini;
pub mod ollama;
mod planner;
mod proposals;
pub mod runtime;
mod secrets;
mod state;
mod work;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = state::Store::open(&data_dir)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            work::normalize_store(&store)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let gemini = gemini::GeminiCredentials::new()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(store);
            app.manage(gemini);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            state::load_work_state,
            state::load_accessibility_preferences,
            state::set_accessibility_preferences,
            planner::plan_remaining_work,
            planner::replan_remaining_work,
            work::capture_item,
            work::classify_inbox_item,
            work::start_current_work,
            work::defer_current_work,
            work::resume_deferred_task,
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
