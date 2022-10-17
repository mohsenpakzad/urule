import { invoke } from '@tauri-apps/api';
import { Location, Process } from 'src/models/core';
import { ScanInfo, ScanType, ValueType } from 'src/models/scan';

export function useUruleCore() {
  let currentValueType = '';

  async function getProcesses() {
    return await invoke<Process[]>('get_processes');
  }

  async function getOpenedProcess() {
    return await invoke<Process>('get_opened_process');
  }

  async function clearLastScan() {
    return await invoke<void>('clear_last_scan');
  }

  async function writeOpenedProcessMemory(address: number, value: number) {
    return await invoke<number | null>(
      `write_opened_process_memory_${currentValueType}`,
      {
        address,
        value,
      }
    );
  }

  async function getLastScan(limit: number, offset: number) {
    return await invoke<[number, Location[]]>(
      `get_last_scan_${currentValueType}`,
      {
        limit,
        offset,
      }
    );
  }

  async function firstScan(
    pid: number,
    valueType: ValueType,
    scanInfo: ScanInfo
  ) {
    currentValueType = valueType.toLowerCase();
    await invoke<void>(`first_scan_${currentValueType}`, {
      pid,
      valueType,
      scanInfo: deleteUnnecessaryValues(scanInfo),
    });
  }

  async function nextScan(scanInfo: ScanInfo) {
    await invoke<void>(`next_scan_${currentValueType}`, {
      scanInfo: deleteUnnecessaryValues(scanInfo),
    });
  }

  async function undoScan() {
    await invoke<void>(`undo_scan_${currentValueType}`);
  }

  function deleteUnnecessaryValues(scanInfo: ScanInfo) {
    switch (scanInfo.typ) {
      case ScanType.Unknown:
      case ScanType.Unchanged:
      case ScanType.Changed:
      case ScanType.Decreased:
      case ScanType.Increased:
        delete scanInfo.value;
        break;

      case ScanType.Exact:
      case ScanType.DecreasedBy:
      case ScanType.IncreasedBy:
        delete scanInfo.value?.Range;
        break;

      case ScanType.InRange:
        delete scanInfo.value?.Exact;
    }
    return scanInfo;
  }

  return {
    getProcesses,
    getOpenedProcess,
    clearLastScan,
    writeOpenedProcessMemory,
    getLastScan,
    firstScan,
    nextScan,
    undoScan,
  };
}
