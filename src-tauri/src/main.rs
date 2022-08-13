#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod process;
mod scan;

use std::sync::Mutex;
use process::{Process, ProcessItem};
use scan::{Scan, Scannable, Region};
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
        .invoke_handler(tauri::generate_handler![get_processes, first_scan, next_scan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_processes() -> Vec<ProcessItem> {
    process::enum_proc()
        .unwrap()
        .into_iter()
        .flat_map(Process::open)
        .flat_map(|proc| match proc.name() {
            Ok(name) => Ok(ProcessItem {
                pid: proc.pid(),
                name,
            }),
            Err(err) => Err(err),
        })
        .collect::<Vec<_>>()
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
    let scan = scan_str.parse::<Scan<Box<dyn Scannable>>>().unwrap();
    let mut last_scan = process.scan_regions(&regions, scan);
    println!(
        "Found {} locations",
        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
    );
    *state.opened_process.lock().unwrap() = Some(process);
    *state.last_scan.lock().unwrap() = last_scan;
}

#[tauri::command]
fn next_scan(scan_str: String, state: tauri::State<AppState>) {
    let scan = scan_str.parse::<Scan<Box<dyn Scannable>>>().unwrap();
    let last_scan = state.opened_process.lock().unwrap().as_ref().unwrap().rescan_regions(&state.last_scan.lock().unwrap(), scan);
    println!(
        "Now have {} locations",
        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
    );
    *state.last_scan.lock().unwrap() = last_scan;
}

