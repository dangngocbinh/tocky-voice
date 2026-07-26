//! Tocky Voice — realtime speech to text with an optional AI cleanup pass,
//! pasted straight into whatever app has focus.

// `audio`, `settings`, `stt` and `refine` are public so the integration tests in
// `tests/` can drive the provider protocols directly against the live APIs.
pub mod audio;
pub mod refine;
pub mod settings;
pub mod stt;

mod commands;
mod focus;
mod history;
mod hotkeys;
mod inject;
mod overlay;
mod private_file;
mod session;
mod tray;
mod state;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    hotkeys::on_shortcut(app, shortcut, event.state());
                })
                .build(),
        )
        .manage(session::Recorder::default())
        .manage(hotkeys::HotkeyRegistry::default())
        .setup(|app| {
            let handle = app.handle().clone();

            // Menu-bar app: this is what keeps the overlay from stealing focus from the
            // app being dictated into, which would send the paste keystroke to us instead.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let settings = settings::load(&handle);
            if let Ok(dir) = handle.path().app_data_dir() {
                settings::secrets::configure(dir, settings.use_os_keychain);
            }
            log::info!(
                "starting: stt={:?} llm={}/{} mode={} accessibility={}",
                settings.stt.provider,
                settings.llm.preset,
                settings.llm.model,
                settings.active_mode_id,
                inject::can_synthesize_input(),
            );
            log::info!(
                "credential store: {}",
                if settings.use_os_keychain {
                    "OS keychain"
                } else {
                    "local vault (0600)"
                }
            );
            app.manage(state::AppState::new(settings.clone()));
            hotkeys::apply(&handle, &settings);
            tray::build(&handle, &settings)?;

            // Under the Accessory activation policy the app never activates on its own,
            // so the settings window would open behind everything else. Ask for focus
            // explicitly at startup — without this the app looks like it launched into
            // nothing but a menu-bar icon.
            show_settings_window(&handle);

            // Ask for Accessibility up front. Without it there is no paste and no
            // hold-a-modifier push-to-talk, and both fail silently — the system prompt
            // with its "Open System Settings" button is far better than a log line.
            #[cfg(target_os = "macos")]
            if !hotkeys::macos_ptt::prompt_for_accessibility_permission() {
                log::warn!("Accessibility permission is not granted yet");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the settings window should leave the app running in the menu bar.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::reset_settings,
            commands::list_llm_presets,
            commands::list_input_devices,
            commands::set_api_key,
            commands::delete_api_key,
            commands::key_status,
            commands::start_recording,
            commands::stop_recording,
            commands::cancel_recording,
            commands::toggle_recording,
            commands::set_active_mode,
            commands::get_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::copy_text,
            commands::permission_status,
            commands::open_accessibility_settings,
            commands::open_url,
            commands::test_llm,
            commands::list_models,
            commands::show_main_window,
            commands::suspend_hotkeys,
            commands::resume_hotkeys,
        ])
        .build(tauri::generate_context!())
        .expect("error while starting Tocky Voice")
        .run(|app, event| {
            // Clicking the app again in Finder or the Dock sends Reopen rather than
            // launching a second copy. A menu-bar app has to answer it itself, or the
            // relaunch looks like the app silently refused to open.
            if let tauri::RunEvent::Reopen { .. } = event {
                show_settings_window(app);
            }
        });
}

/// Brings the settings window to the front. The app runs as a menu-bar accessory, so
/// it never activates on its own — without this the window opens behind everything,
/// or a relaunch from Finder appears to do nothing at all.
///
/// Deliberately does *not* switch the activation policy: flipping to `Regular` and
/// back makes the window vanish when the policy reverts.
pub fn show_settings_window(app: &tauri::AppHandle) {
    // Un-hides the whole app on macOS; a no-op elsewhere.
    #[cfg(target_os = "macos")]
    let _ = app.show();

    let Some(window) = app.get_webview_window("main") else {
        log::error!("main window is missing");
        return;
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

