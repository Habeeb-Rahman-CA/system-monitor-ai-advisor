use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Components, Disks, Networks, System};
use tauri::State;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_space: u64,
    available_space: u64,
    kind: String,
}

#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory: u64,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_cores: usize,
    physical_cores: usize,
    cpu_model: String,
    cpu_arch: String,
    cpu_freq: u64,
    cpus: Vec<f32>,
    cpu_temp: Option<f32>,
    memory_used: u64,
    memory_total: u64,
    os_name: String,
    os_version: String,
    uptime: u64,
    disks: Vec<DiskInfo>,
    net_received: u64,
    net_transmitted: u64,
    processes: Vec<ProcessInfo>,
    gpu_name: String,
    battery_level: Option<f32>,
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
    let physical_cores = sys.physical_core_count().unwrap_or(cpu_cores);
    let cpu_model = sys
        .cpus()
        .get(0)
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_arch = System::cpu_arch();
    let cpu_freq = sys.cpus().get(0).map(|c| c.frequency()).unwrap_or(0);

    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();

    // Disks metrics
    let disks_info = Disks::new_with_refreshed_list();
    let disks = disks_info
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            kind: format!("{:?}", disk.kind()),
        })
        .collect();

    // Network metrics
    let networks = Networks::new_with_refreshed_list();
    let mut net_received = 0;
    let mut net_transmitted = 0;
    for (_interface_name, data) in &networks {
        net_received += data.total_received();
        net_transmitted += data.total_transmitted();
    }

    let cpus: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    let components = Components::new_with_refreshed_list();
    let mut cpu_temp = None;
    let mut battery_level = None;
    let mut gpu_name = "Integrated/N/A".to_string();

    for c in &components {
        let label = c.label().to_lowercase();
        // Temperature check
        if cpu_temp.is_none()
            && (label.contains("cpu") || label.contains("package") || label.contains("core"))
        {
            cpu_temp = c.temperature();
        }

        // Battery check
        if label.contains("battery") {
            // Components often store current capacity or level in some forms
            // sysinfo doesn't directly expose level, but sometimes temperature or other things
            // We'll leave it as None if not found
        }

        // GPU check in components
        if label.contains("gpu")
            || label.contains("nvidia")
            || label.contains("amd")
            || label.contains("intel hd")
        {
            gpu_name = c.label().to_string();
        }
    }

    // Processes metrics
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            name: process.name().to_string_lossy().into_owned(),
            pid: pid.as_u32(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect();

    // Sort by CPU usage descending
    processes.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(10);

    SystemStats {
        cpu_usage,
        cpu_cores,
        physical_cores,
        cpu_model,
        cpu_arch,
        cpu_freq,
        cpus,
        cpu_temp,
        memory_used,
        memory_total,
        os_name,
        os_version,
        uptime,
        disks,
        net_received,
        net_transmitted,
        processes,
        gpu_name,
        battery_level,
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
