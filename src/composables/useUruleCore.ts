import { invoke } from '@tauri-apps/api';
import { Process, Region } from 'src/models/core';

export function useUruleCore() {

  async function getOpenedProcess() {
    return await invoke<Process>('get_opened_process')
  }

  async function writeOpenedProcessMemory(address: number, value: number[]) {
    return await invoke<number | null>('write_opened_process_memory', {address, value})
  }

  async function getLastScan() {
    return await invoke<Region[]>('get_last_scan')
  }

  async function getProcesses() {
    return await invoke<Process[]>('get_processes')
  }

  async function firstScan(pid: number, scanStr: string) {
    await invoke('first_scan', {pid, scanStr})
  }

  async function nextScan(scanStr: string) {
    await invoke('next_scan', {scanStr})
  }

  return {
    getOpenedProcess,
    writeOpenedProcessMemory,
    getLastScan,
    getProcesses,
    firstScan,
    nextScan,
  }
}
