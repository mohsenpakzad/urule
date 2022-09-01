import { invoke } from '@tauri-apps/api';
import { ProcessItem } from 'src/models';

export function useUruleCore() {

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
    getProcesses,
    firstScan,
    nextScan,
  }
}
