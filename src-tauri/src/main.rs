#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::error;
use std::io::Write;
mod menu;

#[tauri::command]
async fn read_file(path: String) -> Result<(Vec<u8>, String), String> {
    use std::fs;

    let content = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mime = infer::get(&content)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok((content, mime))
}

#[tauri::command]
async fn save_gif(path: String, data: Vec<u8>) -> Result<String, String> {
    match std::fs::File::create(&path) {
        Ok(mut file) => match file.write_all(&data) {
            Ok(_) => Ok(path.to_string()),
            Err(e) => Err(format!("Failed to write data to file: {}", e)),
        },
        Err(e) => Err(format!("Failed to create file: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let menu_handle = handle.clone();
            match menu::create_menu(&menu_handle) {
                Ok(menu) => {
                    if let Err(e) = menu_handle.set_menu(menu) {
                        error!("Failed to set menu: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to create menu: {}", e);
                }
            }
            menu_handle.on_menu_event(move |app, event| {
                menu::handle_menu_event(app, event.id().as_ref());
            });
            Ok(())
        })
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![read_file, save_gif])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
