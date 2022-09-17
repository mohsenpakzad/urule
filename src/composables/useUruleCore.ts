import { invoke } from '@tauri-apps/api';
import { ProcessItem, Region } from 'src/models';

export function useUruleCore() {

  async function getOpenedProcess() {
    return await invoke<ProcessItem>('get_opened_process')
  }

  async function getLastScan() {
    return await invoke<Region[]>('get_last_scan')
  }

  async function getProcesses() {
    return await invoke<ProcessItem[]>('get_processes')
  }

  async function firstScan(pid: number, scanStr: string) {
    await invoke('first_scan', {pid, scanStr})
  }

  async function nextScan(scanStr: string) {
    await invoke('next_scan', {scanStr})
  }

  return {
    getOpenedProcess,
    getLastScan,
    getProcesses,
    firstScan,
    nextScan,
  }
}
