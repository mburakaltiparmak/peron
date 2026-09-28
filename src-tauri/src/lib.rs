mod autostart;
mod commands;
mod feedback;
mod history_store;
pub mod log;
mod monitor;
mod notify;
mod settings;
mod state;
mod tray;

// GUI-independent code lives in peron-core (shared with peron-cli); re-exported so the app keeps
// using `crate::model`, `crate::proc`, … paths.
pub use peron_core::{classify, error, export, history, i18n, model, net, platform, proc, util};

use tauri::{Manager, RunEvent, WindowEvent};

use crate::util::LockExt;

use crate::state::AppState;

const MINIMIZED_ARG: &str = "--minimized";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == proc::elevate::RELAUNCH_ARG) {
        // Let the non-elevated instance exit and release the single-instance lock.
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
    let start_minimized = autostart::launched_at_login(&args, MINIMIZED_ARG);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app)
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            // The Run-key value name must stay "Peron": installer-hooks.nsh deletes it on uninstall.
            tauri_plugin_autostart::Builder::new()
                .app_name("Peron")
                .arg(MINIMIZED_ARG)
                .build(),
        )
        .setup(move |app| {
            let handle = app.handle();
            log::init(handle);
            app.manage(AppState::new(settings::load(handle)));
            *app.state::<AppState>().history.lock_safe() = history_store::load(handle);
            notify::register(handle);
            tray::create(handle)?;
            // Autostart runs tray-only: no WebView is created until the user opens the window.
            if !start_minimized {
                tray::show_main(handle);
            }
            monitor::spawn(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let close_to_tray = window
                    .state::<AppState>()
                    .settings
                    .lock_safe()
                    .close_to_tray;
                if close_to_tray {
                    // Destroy (not hide) so WebView2 exits; see tray::hide_main.
                    api.prevent_close();
                    tray::hide_main(window.app_handle());
                } else {
                    window.app_handle().exit(0);
                }
            }
            WindowEvent::Focused(true) => {
                // Focus (also on restore from minimized) → switch to the active scan pace now.
                window.state::<AppState>().wake_monitor();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::get_process_details,
            commands::kill_process,
            commands::get_settings,
            commands::save_settings,
            commands::get_app_info,
            commands::hide_to_tray,
            commands::window_shown,
            commands::feedback_reserve,
            commands::open_log_folder,
            commands::export_snapshot,
            commands::get_history,
            commands::clear_history,
            commands::export_history,
            commands::remote_list,
            commands::remote_test,
            commands::remote_kill,
            commands::relaunch_as_admin,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        // The last window closing must not end the app; only an explicit exit (tray "Quit",
        // close with close-to-tray off, relaunch) passes an exit code.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
