#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod process;
mod region;
mod scan;

use crate::scan::scan_meta::IntoScan;
use paste::paste;
use process::{Process, ProcessView};
use region::Region;
use scan::scan_meta::{ScanInfo, ValueType};
use std::sync::Mutex;
use winapi::um::winnt;

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

macro_rules! impl_scan {
    ( $( $type:ty : $type_size:expr ),+ ) => {
        paste! {
            pub struct AppState {
                opened_process: Mutex<Option<Process>>,
                scan_value_type: Mutex<ValueType>,
                $([<last_scan_ $type>]: Mutex<Vec<Region<$type_size, $type>>>,)+
            }

            impl AppState {
                fn new() -> Self {
                    AppState {
                        opened_process: Mutex::new(None),
                        scan_value_type: Mutex::new(ValueType::I32),
                        $([<last_scan_ $type>]: Mutex::new(Vec::new()),)+
                    }
                }
            }

            fn main() {
                tauri::Builder::default()
                    .manage(AppState::new())
                    .invoke_handler(tauri::generate_handler![
                        get_processes,
                        get_opened_process,
                        clear_last_scan,
                        $(
                            [<write_opened_process_memory_ $type>],
                            [<get_last_scan_ $type>],
                            [<first_scan_ $type>],
                            [<next_scan_ $type>],
                        )+
                    ])
                    .run(tauri::generate_context!())
                    .expect("error while running tauri application");
            }

            #[tauri::command]
            fn clear_last_scan(state: tauri::State<AppState>) {
                $(state.[<last_scan_ $type>].lock().unwrap().clear();)+
            }

            $(
                #[tauri::command]
                fn [<write_opened_process_memory_ $type>](
                    address: usize,
                    value: $type,
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
                fn [<get_last_scan_ $type>](state: tauri::State<AppState>) -> Vec<Region<$type_size, $type>> {
                    state.[<last_scan_ $type>].lock().unwrap().clone()
                }

                #[tauri::command]
                fn [<first_scan_ $type>](pid: u32, value_type: ValueType, scan_info: ScanInfo, state: tauri::State<AppState>) {
                    let process = Process::open(pid).unwrap();
                    println!("Opened process {:?}", process);
                    println!("****FirstScan****\nValueType: {:?}, ScanInfo: {:?}", value_type, scan_info);

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
                    let scan = scan_info.to_scan(&value_type).unwrap();
                    let last_scan = process.scan_regions(&regions, scan);
                    println!(
                        "Found {} locations",
                        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
                    );
                    *state.opened_process.lock().unwrap() = Some(process);
                    *state.scan_value_type.lock().unwrap() = value_type;
                    *state.[<last_scan_ $type>].lock().unwrap() = last_scan;
                }

                #[tauri::command]
                fn [<next_scan_ $type>](scan_info: ScanInfo, state: tauri::State<AppState>) {
                    println!(
                        "****NextScan****\nValueType: {:?}, ScanInfo: {:?}",
                        &state.scan_value_type.lock().unwrap(), scan_info
                    );
                    let scan = scan_info
                        .to_scan(&state.scan_value_type.lock().unwrap())
                        .unwrap();
                    let last_scan = state
                        .opened_process
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .rescan_regions(&state.[<last_scan_ $type>].lock().unwrap(), scan);
                    println!(
                        "Now have {} locations",
                        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
                    );
                    *state.[<last_scan_ $type>].lock().unwrap() = last_scan;
                }
            )+
        }
    }
}

impl_scan!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8);
