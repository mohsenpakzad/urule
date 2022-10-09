#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod process;
mod region;
mod scan;

use process::{Process, ProcessView};
use region::Region;
use std::sync::Mutex;
use winapi::um::winnt;

pub struct AppState {
    opened_process: Mutex<Option<Process>>,
    last_scan: Mutex<Vec<Region>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            opened_process: Mutex::new(None),
            last_scan: Mutex::new(Vec::new()),
        }
    }

    fn clear(&mut self) {
        self.last_scan.lock().unwrap().clear();
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_processes,
            get_opened_process,
            write_opened_process_memory,
            get_last_scan,
            first_scan,
            next_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_processes() -> Vec<ProcessView> {
    process::enum_proc()
        .unwrap()
        .into_iter()
        .flat_map(Process::open)
        .flat_map(|proc| match proc.name() {
            Ok(name) => Ok(ProcessView {
                pid: proc.pid(),
                name,
            }),
            Err(err) => Err(err),
        })
        .collect::<Vec<_>>()
}

#[tauri::command]
fn get_opened_process(state: tauri::State<AppState>) -> Option<ProcessView> {
    state
        .opened_process
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .try_into()
        .ok()
}

#[tauri::command]
fn write_opened_process_memory(
    address: usize,
    value: i32,
    state: tauri::State<AppState>,
) -> Option<usize> {
    state
        .opened_process
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .write_memory(address, &value)
        .ok()
}

#[tauri::command]
fn get_last_scan(state: tauri::State<AppState>) -> Vec<Region> {
    state.last_scan.lock().unwrap().clone()
}

#[tauri::command]
fn first_scan(pid: u32, scan_str: String, state: tauri::State<AppState>) {
    let process = Process::open(pid).unwrap();
    println!("Opened process {:?}", process);

    const MASK: u32 = winnt::PAGE_EXECUTE_READWRITE
        | winnt::PAGE_EXECUTE_WRITECOPY
        | winnt::PAGE_READWRITE
        | winnt::PAGE_WRITECOPY;

    let regions = process
        .memory_regions()
        .into_iter()
        .filter(|p| (p.Protect & MASK) != 0)
        .collect::<Vec<_>>();

    println!("Scanning {} memory regions", regions.len());
    let scan = scan_str.parse().unwrap();
    let last_scan = process.scan_regions(&regions, scan);
    println!(
        "Found {} locations",
        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
    );
    *state.opened_process.lock().unwrap() = Some(process);
    *state.last_scan.lock().unwrap() = last_scan;
}

#[tauri::command]
fn next_scan(scan_str: String, state: tauri::State<AppState>) {
    let scan = scan_str.parse().unwrap();
    let last_scan = state
        .opened_process
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .rescan_regions(&state.last_scan.lock().unwrap(), scan);
    println!(
        "Now have {} locations",
        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
    );
    *state.last_scan.lock().unwrap() = last_scan;
}
