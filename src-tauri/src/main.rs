#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::io::Write;

#[tauri::command]
async fn read_file(path: String) -> Result<(Vec<u8>, String), String> {
  use std::fs;

    let content = fs::read(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mime = infer::get(&content)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok((content, mime))
}

#[tauri::command]
async fn save_gif(
    path: String,
    data: Vec<u8>,
) -> Result<String, String> {
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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
                .invoke_handler(tauri::generate_handler![read_file,save_gif])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
