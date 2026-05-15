use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn register(app: &AppHandle, accel: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let parsed: Shortcut = accel.parse().map_err(|e| format!("{e:?}"))?;
    gs.on_shortcut(parsed, |app, _shortcut, event| {
        if event.state() != ShortcutState::Pressed {
            return;
        }
        if let Some(win) = app.get_webview_window("main") {
            match win.is_visible() {
                Ok(true) => { let _ = win.hide(); }
                _ => {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        }
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}
