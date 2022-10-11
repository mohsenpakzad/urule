import { invoke } from '@tauri-apps/api';
import { Address, CandidateLocations, Process } from 'src/models/core';
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

  async function getLastScan() {
    const lastScanLocations = await invoke<CandidateLocations[]>(
      `get_last_scan_${currentValueType}`
    );
    return convertCandidateLocationsToAddresses(lastScanLocations);
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

  function convertCandidateLocationsToAddresses(
    lastScanLocations: CandidateLocations[]
  ) {
    return lastScanLocations.flatMap((candidateLocation) => {
      console.log(candidateLocation);

      if (candidateLocation.KeyValue) {
        return Object.entries(candidateLocation.KeyValue).map(
          ([pointer, value]) => <Address>{ pointer: parseInt(pointer), value }
        );
      } else if (candidateLocation.SameValue) {
        const { locations, value } = candidateLocation.SameValue;
        return locations.map(
          (location) => <Address>{ pointer: location, value }
        );
      } else if (candidateLocation.Range) {
        const { range, step, values } = candidateLocation.Range;
        return values.map(
          (value, index) =>
            <Address>{ pointer: range.start + index * step, value }
        );
      } else if (candidateLocation.Offsetted) {
        const { base, offsets, values } = candidateLocation.Offsetted;
        return values.map(
          (value, index) => <Address>{ pointer: base + offsets[index], value }
        );
      } else if (candidateLocation.Masked) {
        const { base, step, mask, values } = candidateLocation.Masked;
        return mask
          .map((m, index) => [m, base + index * step])
          .filter(([m]) => m)
          .map(([, pointer]) => <Address>{ pointer, value: values.shift() });
      }
      return [];
    });
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
  };
}
