import { invoke } from '@tauri-apps/api';
import { Address, Process, Region } from 'src/models/core';
import { ScanInfo, ValueType } from 'src/models/scan';

export function useUruleCore() {
  let currentValueType = '';

  async function getOpenedProcess() {
    return await invoke<Process>('get_opened_process');
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
    const lastScanRegions = await invoke<Region[]>(
      `get_last_scan_${currentValueType}`
    );
    return convertRegionsToAddresses(lastScanRegions);
  }

  async function getProcesses() {
    return await invoke<Process[]>('get_processes');
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
      scanInfo,
    });
  }

  async function nextScan(scanInfo: ScanInfo) {
    await invoke<void>(`next_scan_${currentValueType}`, {
      scanInfo,
    });
  }

  function convertRegionsToAddresses(lastScanRegions: Region[]) {
    return lastScanRegions.flatMap((region) => {
      console.log(region.locations);

      if (region.locations.SameValue) {
        const locations = region.locations.SameValue.locations;
        const value = region.locations.SameValue.value;

        return locations.map(
          (location) => <Address>{ pointer: location, value }
        );
      } else if (region.locations.KeyValue) {
        return Object.entries(region.locations.KeyValue).map(
          ([pointer, value]) => <Address>{ pointer: parseInt(pointer), value }
        );
      }
      // TODO: handle other cases
      return [];
    });
  }

  return {
    getOpenedProcess,
    writeOpenedProcessMemory,
    getLastScan,
    getProcesses,
    firstScan,
    nextScan,
  };
}
