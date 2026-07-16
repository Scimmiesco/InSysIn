pub mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod state;

use state::AppState;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{Disks, Networks, System};
use tauri::Emitter;
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            sys: Mutex::new(sys),
            disks: Mutex::new(disks),
            networks: Mutex::new(networks),
            last_db_save: Mutex::new(Instant::now()),
        })
        .setup(|app| {
            let h = app.handle();
            let pkg_info = h.package_info();
            let config = h.config();
            let about_meta = AboutMetadata {
                name: Some(pkg_info.name.clone()),
                version: Some(pkg_info.version.to_string()),
                copyright: config.bundle.copyright.clone(),
                authors: config.bundle.publisher.clone().map(|p| vec![p]),
                ..Default::default()
            };

            let sep = || PredefinedMenuItem::separator(h);

            let dark = MenuItem::with_id(h, "theme_dark", "Dark Theme", true, None::<&str>)?;
            let light = MenuItem::with_id(h, "theme_light", "Light Theme", true, None::<&str>)?;
            let theme_sub = Submenu::with_items(h, "Theme", true, &[&dark, &light])?;

            let network_item = MenuItem::with_id(h, "navigate_network", "Network Dashboard", true, None::<&str>)?;

            #[cfg(target_os = "macos")]
            let view_menu = Submenu::with_items(h, "View", true, &[
                &PredefinedMenuItem::fullscreen(h, None)?,
                &sep()?,
                &theme_sub,
                &sep()?,
                &network_item,
            ])?;

            #[cfg(not(target_os = "macos"))]
            let view_menu = Submenu::with_items(h, "View", true, &[
                &theme_sub,
                &sep()?,
                &PredefinedMenuItem::fullscreen(h, None)?,
                &sep()?,
                &network_item,
            ])?;

            let window_menu = Submenu::with_id_and_items(
                h,
                WINDOW_SUBMENU_ID,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(h, None)?,
                    &PredefinedMenuItem::maximize(h, None)?,
                    #[cfg(target_os = "macos")]
                    &sep()?,
                    &PredefinedMenuItem::close_window(h, None)?,
                ],
            )?;

            let help_menu = Submenu::with_id_and_items(
                h,
                HELP_SUBMENU_ID,
                "Help",
                true,
                &[
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::about(h, None, Some(about_meta))?,
                ],
            )?;

            let menu = Menu::with_items(h, &[
                #[cfg(target_os = "macos")]
                &Submenu::with_items(h, pkg_info.name.clone(), true, &[
                    &PredefinedMenuItem::about(h, None, Some(about_meta))?,
                    &sep()?,
                    &PredefinedMenuItem::services(h, None)?,
                    &sep()?,
                    &PredefinedMenuItem::hide(h, None)?,
                    &PredefinedMenuItem::hide_others(h, None)?,
                    &sep()?,
                    &PredefinedMenuItem::quit(h, None)?,
                ])?,
                &Submenu::with_items(h, "File", true, &[
                    &PredefinedMenuItem::close_window(h, None)?,
                    #[cfg(not(target_os = "macos"))]
                    &PredefinedMenuItem::quit(h, None)?,
                ])?,
                &Submenu::with_items(h, "Edit", true, &[
                    &PredefinedMenuItem::undo(h, None)?,
                    &PredefinedMenuItem::redo(h, None)?,
                    &sep()?,
                    &PredefinedMenuItem::cut(h, None)?,
                    &PredefinedMenuItem::copy(h, None)?,
                    &PredefinedMenuItem::paste(h, None)?,
                    &sep()?,
                    &PredefinedMenuItem::select_all(h, None)?,
                ])?,
                &view_menu,
                &window_menu,
                &help_menu,
            ])?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "theme_dark" => { let _ = app.emit("theme-changed", "dark"); }
                "theme_light" => { let _ = app.emit("theme-changed", "light"); }
                "navigate_network" => { let _ = app.emit("navigate", "/network"); }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::hardware::ler_hardware,
            commands::historico::obter_historico,
            commands::historico::obter_processos_agrupados,
            commands::network::ler_rede,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
