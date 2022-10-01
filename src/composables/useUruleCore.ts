import { invoke } from '@tauri-apps/api';
import { Address, Process, Region } from 'src/models/core';

export function useUruleCore() {

  async function getOpenedProcess() {
    return await invoke<Process>('get_opened_process')
  }

  async function writeOpenedProcessMemory(address: number, value: number[]) {
    return await invoke<number | null>('write_opened_process_memory', {address, value})
  }

  async function getLastScan() {
    const lastScanRegions = await invoke<Region[]>('get_last_scan')
    return convertRegionsToAddresses(lastScanRegions)
  }

  async function getProcesses() {
    return await invoke<Process[]>('get_processes')
  }

  async function firstScan(pid: number, scanStr: string) {
    await invoke<void>('first_scan', {pid, scanStr})
  }

  async function nextScan(scanStr: string) {
    await invoke<void>('next_scan', {scanStr})
  }

  function convertRegionsToAddresses(lastScanRegions: Region[]) {
    return lastScanRegions.flatMap(region => {

      if (region.locations.Discrete) {
        const locations = region.locations.Discrete!.locations;
        const value = byteArrayToLong(region.value.Exact!)

        return locations.map(location => <Address>{pointer: location, value})
      }
      // TODO: handle other cases
      return []
    })
  }

  function longToByteArray(/*long*/long: number) {
    // we want to represent the input as a 8-bytes array
    const byteArray = [0, 0, 0, 0, 0, 0, 0, 0];

    for (let index = 0; index < byteArray.length; index++) {
      const byte = long & 0xff;
      byteArray [index] = byte;
      long = (long - byte) / 256;
    }

    return byteArray;
  }

  function byteArrayToLong(/*byte[]*/byteArray: number[]) {
    let value = 0;
    for (let i = byteArray.length - 1; i >= 0; i--) {
      value = (value * 256) + byteArray[i];
    }

    return value;
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
