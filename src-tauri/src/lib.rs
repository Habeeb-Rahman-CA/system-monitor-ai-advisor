use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;
use tauri::State;

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    os_name: String,
    os_version: String,
}

pub struct AppState {
    sys: Mutex<System>,
}

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());

    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        os_name,
        os_version,
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            sys: Mutex::new(System::new_all()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
