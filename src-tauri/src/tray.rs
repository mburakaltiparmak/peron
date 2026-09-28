//! System tray and main-window lifecycle.
//!
//! Hiding to the tray *destroys* the window so WebView2's processes exit (~200 MB); showing it
//! again recreates it from config. Clicking a port in the menu only asks the UI to open its kill
//! confirmation (never kills directly).

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder, Wry};

use crate::i18n::{self, Lang, Text};
use crate::model::PortEntry;
use crate::settings;
use crate::state::{now_ms, AppState};
use crate::util::LockExt;

const TRAY_ID: &str = "main";
const MAIN: &str = "main";
pub const REQUEST_KILL: &str = "request-kill";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let lang = app.state::<AppState>().settings.lock_safe().lang();
    let menu = build_menu(app, lang, &[])?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Peron")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "quit" => app.exit(0),
            "lang:tr" => set_language(app, Lang::Tr),
            "lang:en" => set_language(app, Lang::En),
            id => {
                if let Some(entry_id) = id.strip_prefix("kill:") {
                    show_main(app);
                    let _ = app.emit(REQUEST_KILL, entry_id);
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn set_language(app: &AppHandle, lang: Lang) {
    let mut next = app.state::<AppState>().settings.lock_safe().clone();
    next.language = Some(lang);
    if let Err(e) = settings::apply(app, next) {
        crate::log_error!("language change failed: {e}");
    }
}

/// Top dev listeners by open duration, one per process+port.
fn top_dev_ports(entries: &[PortEntry]) -> Vec<&PortEntry> {
    let mut seen = std::collections::HashSet::new();
    let mut devs: Vec<&PortEntry> = entries
        .iter()
        .filter(|e| e.is_dev && !e.is_system && e.is_listening())
        .filter(|e| seen.insert((e.pid, e.local_port)))
        .collect();
    devs.sort_by_key(|e| e.opened_at_ms.unwrap_or(i64::MAX));
    devs.truncate(5);
    devs
}

fn item(app: &AppHandle, id: &str, text: String, enabled: bool) -> tauri::Result<MenuItem<Wry>> {
    MenuItem::with_id(app, id, text, enabled, None::<&str>)
}

fn build_menu(app: &AppHandle, lang: Lang, devs: &[(String, String)]) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&item(app, "open", i18n::t(lang, Text::OpenApp), true)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    if devs.is_empty() {
        menu.append(&item(app, "none", i18n::t(lang, Text::NoDevPorts), false)?)?;
    } else {
        menu.append(&item(
            app,
            "header",
            i18n::t(lang, Text::PickToClose),
            false,
        )?)?;
        for (id, label) in devs {
            menu.append(&item(app, &format!("kill:{id}"), label.clone(), true)?)?;
        }
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let tr = CheckMenuItem::with_id(
        app,
        "lang:tr",
        "Türkçe",
        true,
        lang == Lang::Tr,
        None::<&str>,
    )?;
    let en = CheckMenuItem::with_id(
        app,
        "lang:en",
        "English",
        true,
        lang == Lang::En,
        None::<&str>,
    )?;
    menu.append(&Submenu::with_items(
        app,
        i18n::t(lang, Text::Language),
        true,
        &[&tr, &en],
    )?)?;
    menu.append(&item(app, "quit", i18n::t(lang, Text::Quit), true)?)?;
    Ok(menu)
}

pub fn update(app: &AppHandle, entries: &[PortEntry]) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let state = app.state::<AppState>();
    let lang = state.settings.lock_safe().lang();

    let mut listening: Vec<(u32, u16)> = entries
        .iter()
        .filter(|e| e.is_listening() && !e.is_system)
        .map(|e| (e.pid, e.local_port))
        .collect();
    listening.sort_unstable();
    listening.dedup();
    let _ = tray.set_tooltip(Some(i18n::t(lang, Text::Tooltip(listening.len()))));

    let now = now_ms();
    let devs: Vec<(String, String)> = top_dev_ports(entries)
        .into_iter()
        .map(|e| {
            let age = e
                .opened_at_ms
                .map(|t| format!(" · {}", i18n::duration(lang, now - t)))
                .unwrap_or_default();
            (
                e.id.clone(),
                format!("{} :{}{}", e.process_name, e.local_port, age),
            )
        })
        .collect();

    let signature = format!(
        "{lang:?}\n{}",
        devs.iter()
            .map(|(_, l)| l.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut last = state.tray_signature.lock_safe();
    if *last == signature {
        return;
    }
    if let Ok(menu) = build_menu(app, lang, &devs) {
        let _ = tray.set_menu(Some(menu));
        *last = signature;
    }
}

/// Rebuilds tooltip and menu now (e.g. after a language change).
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.tray_signature.lock_safe().clear();
    let entries = state.snapshot.lock_safe().clone();
    update(app, &entries);
}

fn create_main(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == MAIN)
        .cloned()
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    // Created hidden; the UI shows it after its first render to avoid a blank flash.
    // Title bar in the chosen theme from the first frame (not only after the UI loads).
    let theme = app
        .state::<AppState>()
        .settings
        .lock_safe()
        .theme
        .window_theme();
    let window = WebviewWindowBuilder::from_config(app, &config)?
        .theme(theme)
        .build()?;
    let fallback = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(2000));
        if !fallback.is_visible().unwrap_or(true) {
            let _ = fallback.show();
            let _ = fallback.set_focus();
        }
    });
    Ok(window)
}

pub fn show_main(app: &AppHandle) {
    match app.get_webview_window(MAIN) {
        Some(w) => {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
        }
        None => {
            if let Err(e) = create_main(app) {
                crate::log_error!("window creation failed: {e}");
            }
        }
    }
    app.state::<AppState>().wake_monitor();
}

/// Closes the window to the tray, releasing the WebView.
pub fn hide_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.destroy();
    }
    app.state::<AppState>().wake_monitor();
}

fn toggle_main(app: &AppHandle) {
    let visible = app
        .get_webview_window(MAIN)
        .is_some_and(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false));
    if visible {
        hide_main(app);
    } else {
        show_main(app);
    }
}
