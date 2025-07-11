use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_dialog::DialogExt;

pub fn create_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, tauri::Error> {
    let custom_about_metadata = AboutMetadata {
        authors: Some(vec!["Cavazza Tommaso <contact@cavazzatommaso.com>".to_string()]),
        comments: Some("A simple cross-platform desktop app to create GIFs from image sequences".to_string()),
        copyright: Some("© 2025".to_string()),
        license: Some("MIT".to_string()),
        website: Some("https://cavazzatommaso.com".to_string()),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, "Looply")
        .item(&PredefinedMenuItem::about(app, None, Some(custom_about_metadata))?)
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

pub fn handle_menu_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "report_issue" => {
            app.dialog()
                .message("If you encounter any issues, please email us at contact@cavazzatommaso.com")
                .title("Report Issue")
                .show(|_| {});
            
        }
        "check_for_update" => {
            let _ = app.emit("check-update", {});
        }
        _ => {}
    }
}
