use crate::poller;
use crate::store::{HudMode, Store};
use std::sync::Arc;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

pub fn build(app: &AppHandle, store: Arc<Store>) -> tauri::Result<()> {
    let current = store.settings().mode;

    let mi_minimal  = CheckMenuItem::with_id(app, "mode_minimal", "Minimal", true, current == HudMode::Minimal, None::<&str>)?;
    let mi_full     = CheckMenuItem::with_id(app, "mode_full",    "Full",    true, current == HudMode::Full, None::<&str>)?;
    let mi_toggle   = MenuItem::with_id(app, "toggle", "Show / hide window", true, None::<&str>)?;
    let mi_recenter = MenuItem::with_id(app, "recenter", "Reset position", true, None::<&str>)?;
    let mi_refresh  = MenuItem::with_id(app, "refresh", "Refresh now", true, None::<&str>)?;
    let mi_quit     = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep         = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[
        &mi_minimal, &mi_full,
        &sep,
        &mi_toggle,
        &mi_recenter,
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
                "recenter" => {
                    let mode = store_for_menu.settings().mode;
                    recenter_window(app, &store_for_menu, mode);
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

fn window_size(mode: HudMode) -> (u32, u32) {
    match mode {
        HudMode::Minimal => (220, 130),
        HudMode::Full => (320, 280),
    }
}

pub fn resize_for_mode(app: &AppHandle, mode: HudMode) {
    let (w, h) = window_size(mode);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_size(tauri::PhysicalSize::new(w, h));
    }
}

/// Returns true if the window's center point would lie inside any currently
/// connected monitor. Used to detect stale saved positions after monitor
/// reconfiguration (dock/undock, monitor unplugged, etc.).
pub fn position_visible(win: &WebviewWindow, mode: HudMode, x: i32, y: i32) -> bool {
    let (w, h) = window_size(mode);
    let cx = x + (w / 2) as i32;
    let cy = y + (h / 2) as i32;
    let monitors = match win.available_monitors() {
        Ok(m) => m,
        Err(_) => return true, // can't enumerate — assume on-screen and don't move
    };
    monitors.iter().any(|m| {
        let mp = m.position();
        let ms = m.size();
        cx >= mp.x && cx < mp.x + ms.width as i32
            && cy >= mp.y && cy < mp.y + ms.height as i32
    })
}

/// Move the window to a safe spot near the top-right of the primary monitor,
/// show + focus it, and persist the new position. Used both by the tray
/// "Recenter window" item and as the off-screen fallback at startup.
pub fn recenter_window(app: &AppHandle, store: &Arc<Store>, mode: HudMode) {
    let Some(win) = app.get_webview_window("main") else { return };
    let monitor = match win.primary_monitor() {
        Ok(Some(m)) => m,
        _ => match win.available_monitors() {
            Ok(mut ms) => match ms.drain(..).next() {
                Some(m) => m,
                None => return,
            },
            _ => return,
        },
    };
    let mp = monitor.position();
    let ms = monitor.size();
    let (w, _h) = window_size(mode);
    const PAD: i32 = 24;
    let x = mp.x + ms.width as i32 - w as i32 - PAD;
    let y = mp.y + PAD;
    let _ = win.set_position(PhysicalPosition::new(x, y));
    let _ = win.show();
    let _ = win.set_focus();
    store.set_window_pos(x, y);
}
