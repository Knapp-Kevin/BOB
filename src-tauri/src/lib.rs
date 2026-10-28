mod agent;
mod backup;
mod export;
mod gemini;
pub mod inference_policy;
pub mod inference_transport;
pub mod ollama;
mod openrouter;
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
        // Registered first: a second launch hands off to the running window and exits
        // before setup opens the canonical profile (see state::ProfileLock).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }))
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = match state::Store::open(&data_dir) {
                Ok(store) => store,
                Err(error) => {
                    // A GUI launch has no console, so a busy profile must be reported
                    // visibly rather than as a setup panic on stderr.
                    if let Some(busy) = error.downcast_ref::<state::ProfileInUse>() {
                        report_profile_in_use(busy);
                        std::process::exit(1);
                    }
                    return Err(std::io::Error::other(error.to_string()).into());
                }
            };
            work::normalize_store(&store)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            let gemini = gemini::GeminiCredentials::new()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(store);
            app.manage(gemini);
            app.manage(openrouter::OpenRouterCredentials::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            state::load_work_state,
            state::load_chat_history,
            state::append_chat_message,
            state::clear_chat_history,
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
            openrouter::openrouter_settings,
            openrouter::configure_openrouter_model,
            openrouter::set_openrouter_chat_session,
            openrouter::generate_openrouter,
            openrouter::openrouter_credential_status,
            openrouter::configure_openrouter_credential,
            openrouter::remove_openrouter_credential,
            gemini::generate_gemini_context
        ])
        .run(tauri::generate_context!())
        .expect("error while running B.O.B.");
}

fn focus_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(windows)]
fn report_profile_in_use(busy: &state::ProfileInUse) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let title = wide("B.O.B. is already open");
    let body = wide(&format!(
        "B.O.B. could not start because its data is in use by another program.

{busy}

Nothing was changed."
    ));
    // SAFETY: both buffers are NUL-terminated UTF-16 and outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONWARNING,
        );
    }
}

#[cfg(not(windows))]
fn report_profile_in_use(busy: &state::ProfileInUse) {
    eprintln!("{busy}");
}
