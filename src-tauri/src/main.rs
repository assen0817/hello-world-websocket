// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc;

use tokio_tungstenite::tungstenite::Message;
use websocket::websocket_setup;
mod config;
mod websocket;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn send_websocket(
    app_state: tauri::State<'_, config::AppState>,
    msg: &str
) {
    app_state.sender.send(Message::Text(format!("{}", msg))).unwrap();
}

fn main() {
    // tauriコマンドからwebsocketへの情報を共有に使用
    let (sender, receiver) = mpsc::channel();
    let app_state = config::AppState::new(sender);

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| websocket_setup(app, receiver))
        .invoke_handler(tauri::generate_handler![send_websocket])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
