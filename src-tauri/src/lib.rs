mod commands;
mod credentials;
mod error;
mod hotkey;
mod poller;
mod store;
mod tray;
mod usage_client;

use std::sync::Arc;
use store::Store;
use tauri::{Manager, PhysicalPosition, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();
    let store = Arc::new(Store::load());
    let initial_settings = store.settings();
    let initial_mode = initial_settings.mode;
    let initial_hotkey = initial_settings.hotkey.clone();
    let initial_pos = initial_settings.window_pos;

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(store.clone())
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::set_mode,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            tray::build(&handle, store.clone())?;
            tray::resize_for_mode(&handle, initial_mode);
            if let Err(e) = hotkey::register(&handle, &initial_hotkey) {
                log::warn!("hotkey registration failed: {e}");
            }

            // Restore saved window position and watch for moves.
            if let Some(win) = handle.get_webview_window("main") {
                if let Some((x, y)) = initial_pos {
                    let _ = win.set_position(PhysicalPosition::new(x, y));
                }
                let store_for_move = store.clone();
                win.on_window_event(move |event| {
                    if let WindowEvent::Moved(pos) = event {
                        store_for_move.set_window_pos(pos.x, pos.y);
                    }
                });
            }

            poller::spawn(handle.clone(), store.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
