import { invoke } from '@tauri-apps/api';
import { Address, Process, Region } from 'src/models/core';

export function useUruleCore() {
  async function getOpenedProcess() {
    return await invoke<Process>('get_opened_process');
  }

  async function writeOpenedProcessMemory(address: number, value: number) {
    return await invoke<number | null>('write_opened_process_memory', {
      address,
      value,
    });
  }

  async function getLastScan() {
    const lastScanRegions = await invoke<Region[]>('get_last_scan');
    return convertRegionsToAddresses(lastScanRegions);
  }

  async function getProcesses() {
    return await invoke<Process[]>('get_processes');
  }

  // TODO: add scan config to scan parameters
  async function firstScan(pid: number, scanStr: string) {
    await invoke<void>('first_scan', { pid, scanStr });
  }

  // TODO: add scan config to scan parameters
  async function nextScan(scanStr: string) {
    await invoke<void>('next_scan', { scanStr });
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
