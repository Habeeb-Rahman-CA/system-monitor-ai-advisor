use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Disks, Networks, System};
use tauri::State;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_space: u64,
    available_space: u64,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_cores: usize,
    cpus: Vec<f32>,
    memory_used: u64,
    memory_total: u64,
    os_name: String,
    os_version: String,
    uptime: u64,
    disks: Vec<DiskInfo>,
    net_received: u64,
    net_transmitted: u64,
}

pub struct AppState {
    sys: Mutex<System>,
}

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();

    // System metrics
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_cores = sys.cpus().len();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();

    // Disks metrics (separate struct in sysinfo 0.33)
    let disks_info = Disks::new_with_refreshed_list();
    let disks = disks_info
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        })
        .collect();

    // Network metrics (separate struct in sysinfo 0.33)
    let networks = Networks::new_with_refreshed_list();
    let mut net_received = 0;
    let mut net_transmitted = 0;
    for (_interface_name, data) in &networks {
        net_received += data.total_received();
        net_transmitted += data.total_transmitted();
    }

    let cpus: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    SystemStats {
        cpu_usage,
        cpu_cores,
        cpus,
        memory_used,
        memory_total,
        os_name,
        os_version,
        uptime,
        disks,
        net_received,
        net_transmitted,
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
