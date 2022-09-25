import { Process } from 'src/models/core';

export function useFormatter() {

  const nf = Intl.NumberFormat('en');
  const bigNf = Intl.NumberFormat('en', {  notation: 'scientific' });

  function formatNumber(num: number) {
    if (num > 2 ** 32 || num < (-2) ** 31) {
      return bigNf.format(num);
    }
    return nf.format(num);
  }

  function formatProcess(process: Process) {
    return `${process.name} - ${process.pid}`
  }

  function formatMinMaxValue(min: number, max: number){
    return `Min: ${formatNumber(min)}, Max: ${formatNumber(max)}`
  }

  return {
    formatNumber,
    formatProcess,
    formatMinMaxValue,
  }
}
