//! The menu bar icon and its menu.
//!
//! Carries its own copy of the handful of strings it shows. The frontend dictionary
//! lives in the webview and this menu is drawn by the OS, so there is no way to share
//! one — which is exactly why the tray stayed English after the rest of the app was
//! translated. Keeping the strings next to the menu that uses them makes the omission
//! visible the next time someone adds an item here.

use crate::session;
use crate::settings::AppSettings;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

const TRAY_ID: &str = "fvt-tray";

struct Labels {
    toggle: &'static str,
    settings: &'static str,
    quit: &'static str,
}

const EN: Labels = Labels {
    toggle: "Start / stop dictation",
    settings: "Settings…",
    quit: "Quit Tocky Voice",
};

const VI: Labels = Labels {
    toggle: "Bắt đầu / dừng đọc",
    settings: "Cài đặt…",
    quit: "Thoát Tocky Voice",
};

/// Mirrors the frontend's rule: anything not clearly Vietnamese falls back to English,
/// and the choice is made on language alone — a Vietnamese-speaking user on an English
/// system should not be switched by their country.
fn labels_for(settings: &AppSettings) -> &'static Labels {
    let vietnamese = match settings.ui_language.as_str() {
        "vi" => true,
        "en" => false,
        _ => sys_locale::get_locale()
            .map(|l| l.to_lowercase().starts_with("vi"))
            .unwrap_or(false),
    };
    if vietnamese {
        &VI
    } else {
        &EN
    }
}

fn build_menu(app: &tauri::AppHandle, settings: &AppSettings) -> tauri::Result<Menu<tauri::Wry>> {
    let labels = labels_for(settings);
    let toggle = MenuItem::with_id(app, "toggle", labels.toggle, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", labels.settings, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&toggle, &settings_item, &separator, &quit])
}

pub fn build(app: &tauri::AppHandle, settings: &AppSettings) -> tauri::Result<()> {
    let menu = build_menu(app, settings)?;

    // A dedicated monochrome glyph, not the app icon: `icon_as_template` tells macOS to
    // recolour the image for the current menu bar, so a full-colour icon would collapse
    // into a solid blob.
    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_icon)
        .icon_as_template(true)
        .tooltip("Tocky Voice")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => session::toggle(app),
            "settings" => crate::show_settings_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Redraws the menu in the current language. Called after settings are saved, so
/// switching language updates the tray without a restart, like the rest of the UI.
pub fn apply_language(app: &tauri::AppHandle, settings: &AppSettings) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match build_menu(app, settings) {
        Ok(menu) => {
            if let Err(e) = tray.set_menu(Some(menu)) {
                log::warn!("could not update the tray menu: {e}");
            }
        }
        Err(e) => log::warn!("could not rebuild the tray menu: {e}"),
    }
}
