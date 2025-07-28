use serde::Deserialize;
use std::fs;
use tauri::menu::{
    AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;

#[derive(Deserialize)]
struct FFmpegVersion {
    version: String,
}

fn get_ffmpeg_version<R: Runtime>(app: &AppHandle<R>) -> String {
    // Try to resolve the resource path for the JSON file
    match app.path().resolve(
        "../src/lib/assets/ffmpeg-version.json",
        tauri::path::BaseDirectory::Resource,
    ) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<FFmpegVersion>(&content) {
                Ok(ffmpeg_info) => format!("FFmpeg version: {}", ffmpeg_info.version),
                Err(_) => "FFmpeg version: Unknown".to_string(),
            },
            Err(_) => "FFmpeg version: Unknown".to_string(),
        },
        Err(_) => "FFmpeg version: Unknown".to_string(),
    }
}

pub fn create_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, tauri::Error> {
    let ffmpeg_version = get_ffmpeg_version(app);

    let app_version = app.package_info().version.to_string();

    let custom_about_metadata = AboutMetadata {
        authors: Some(vec![
            "Cavazza Tommaso <contact@cavazzatommaso.com>".to_string(),
        ]),
        comments: Some(format!(
            "A simple cross-platform desktop app to create GIFs from image sequences",
        )),
        version: Some(format!("{}\n({})", app_version, ffmpeg_version)),
        copyright: Some("© 2025".to_string()),
        license: Some("MIT".to_string()),
        website: Some("https://cavazzatommaso.com".to_string()),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, "Looply")
        .item(&PredefinedMenuItem::about(
            app,
            None,
            Some(custom_about_metadata),
        )?)
        .item(&MenuItemBuilder::with_id("check_for_update", "Check for Update").build(app)?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, None).unwrap())
        .item(&PredefinedMenuItem::hide_others(app, None).unwrap())
        .item(&PredefinedMenuItem::show_all(app, None).unwrap())
        .separator()
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None).unwrap())
        .item(&PredefinedMenuItem::redo(app, None).unwrap())
        .separator()
        .item(&PredefinedMenuItem::cut(app, None).unwrap())
        .item(&PredefinedMenuItem::copy(app, None).unwrap())
        .item(&PredefinedMenuItem::paste(app, None).unwrap())
        .item(&PredefinedMenuItem::select_all(app, None).unwrap())
        .build()?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&MenuItemBuilder::with_id("report_issue", "Report Issue").build(app)?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&edit_menu)
        .item(&help_menu)
        .build()?;

    Ok(menu)
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event_id: &str) {
    match event_id {
        "report_issue" => {
            app.dialog()
                .message(
                    "If you encounter any issues, please email us at contact@cavazzatommaso.com",
                )
                .title("Report Issue")
                .show(|_| {});
        }
        "check_for_update" => {
            let _ = app.emit("check-update", {});
        }
        _ => {}
    }
}
