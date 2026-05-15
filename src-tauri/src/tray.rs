use crate::poller;
use crate::store::{HudMode, Store};
use std::sync::Arc;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub fn build(app: &AppHandle, store: Arc<Store>) -> tauri::Result<()> {
    let current = store.settings().mode;

    let mi_minimal = CheckMenuItem::with_id(app, "mode_minimal", "Minimal", true, current == HudMode::Minimal, None::<&str>)?;
    let mi_full    = CheckMenuItem::with_id(app, "mode_full",    "Full",    true, current == HudMode::Full, None::<&str>)?;
    let mi_toggle  = MenuItem::with_id(app, "toggle", "Show / hide window", true, None::<&str>)?;
    let mi_refresh = MenuItem::with_id(app, "refresh", "Refresh now", true, None::<&str>)?;
    let mi_quit    = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep        = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[
        &mi_minimal, &mi_full,
        &sep,
        &mi_toggle,
        &mi_refresh,
        &sep,
        &mi_quit,
    ])?;

    let store_for_menu = store.clone();
    let mi_minimal_h = mi_minimal.clone();
    let mi_full_h = mi_full.clone();
    let sync_checks = move |selected: HudMode| {
        let _ = mi_minimal_h.set_checked(selected == HudMode::Minimal);
        let _ = mi_full_h.set_checked(selected == HudMode::Full);
    };
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("default icon"))
        .tooltip("Claude HUD")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            let pick = |m: HudMode| {
                set_mode(app, &store_for_menu, m);
                sync_checks(m);
            };
            match id {
                "mode_minimal" => pick(HudMode::Minimal),
                "mode_full"    => pick(HudMode::Full),
                "toggle" => {
                    if let Some(win) = app.get_webview_window("main") {
                        match win.is_visible() {
                            Ok(true) => { let _ = win.hide(); }
                            _ => { let _ = win.show(); let _ = win.set_focus(); }
                        }
                    }
                }
                "refresh" => {
                    poller::refresh_now(app.clone(), store_for_menu.clone());
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(win) = tray.app_handle().get_webview_window("main") {
                    match win.is_visible() {
                        Ok(true) => { let _ = win.hide(); }
                        _ => { let _ = win.show(); let _ = win.set_focus(); }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn set_mode(app: &AppHandle, store: &Arc<Store>, mode: HudMode) {
    store.set_mode(mode);
    resize_for_mode(app, mode);
    let _ = app.emit("settings://mode", &mode);
}

pub fn resize_for_mode(app: &AppHandle, mode: HudMode) {
    let (w, h) = match mode {
        HudMode::Minimal => (220u32, 130u32),
        HudMode::Full => (320, 280),
    };
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_size(tauri::PhysicalSize::new(w, h));
    }
}
