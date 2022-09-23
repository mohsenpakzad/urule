import { Process } from 'src/models/core';

export function useFormatter() {

  function formatProcess(process: Process) {
    return `${process.name} - ${process.pid}`
  }

  return {
    formatProcess
  }
}
