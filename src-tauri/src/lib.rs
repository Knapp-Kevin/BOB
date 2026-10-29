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
                    let error = match error.downcast::<state::ProfileInUse>() {
                        Ok(busy) => {
                            // Nothing is managed, so the frontend would fall back to
                            // sample data; keep every window hidden until exit.
                            for window in app.webview_windows().values() {
                                let _ = window.hide();
                            }
                            let handle = app.handle().clone();
                            std::thread::spawn(move || {
                                report_profile_in_use(&busy);
                                exit_failed_startup(&handle);
                            });
                            return Ok(());
                        }
                        Err(error) => error,
                    };
                    eprintln!("B.O.B. canonical state startup failed: {error:#}");
                    show_startup_failure_and_exit(app, "B.O.B. could not open its saved data. No automatic recovery was attempted.");
                    return Ok(());
                }
            };
            if let Err(error) = work::normalize_store(&store) {
                eprintln!("B.O.B. startup state validation failed: {error:#}");
                show_startup_failure_and_exit(app, "B.O.B. could not validate its saved work. No automatic recovery was attempted.");
                return Ok(());
            }
            let gemini = gemini::GeminiCredentials::new()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(store);
            app.manage(gemini);
            app.manage(openrouter::OpenRouterCredentials::new());
            // The window is created hidden (tauri.conf.json) and shown only once the
            // canonical Store is open and valid, so a failed startup never flashes it.
            if let Some(window) = app.get_webview_window("main") {
                window.show()?;
            }
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

// AppHandle::exit drops its code (the event loop ends with a plain Exit), so a
// fail-closed startup exits directly with a non-zero status after Tauri cleanup.
fn exit_failed_startup(handle: &tauri::AppHandle) -> ! {
    handle.cleanup_before_exit();
    std::process::exit(1);
}

// Fail closed before the frontend can render sample fixtures without a valid
// canonical Store. Details remain in diagnostic stderr, not in the GUI alert.
fn show_startup_failure_and_exit(app: &mut tauri::App, message: &'static str) {
    for window in app.webview_windows().values() {
        let _ = window.hide();
    }
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        report_startup_failure(message);
        exit_failed_startup(&handle);
    });
}

#[cfg(windows)]
fn report_startup_failure(message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let title = wide("B.O.B. cannot start");
    let body = wide(message);
    // SAFETY: both buffers are NUL-terminated UTF-16 and live through the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn report_startup_failure(message: &str) {
    eprintln!("{message}");
}

fn focus_main_window(app: &tauri::AppHandle) {
    // A failed startup has no Store and is waiting on its error dialog; showing its
    // window would reveal the frontend's sample fixtures instead of saved work.
    if app.try_state::<state::Store>().is_none() {
        return;
    }
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
