import { useFormatter } from 'src/composables/useFormatter';

export function useRules() {
  const formatter = useFormatter();

  function ruleRequired(value: never) {
    return !!value || 'Required';
  }

  function ruleBetween(min: number, max: number) {
    return (value: number) =>
      (value >= min && value <= max) ||
      `Value should between ${formatter.formatNumber(
        min
      )} and ${formatter.formatNumber(max)}`;
  }

  function ruleInteger(value: string) {
    return /^[-+]?\d+$/.test(value) || 'Invalid integer';
  }

  function ruleDecimal(value: string) {
    return /^[+-]?((\d+(\.\d*)?)|(\.\d+))$/.test(value) || 'Invalid decimal';
  }

  function ruleSmaller(max: number) {
    return (value: number) =>
      value < max || `Value should smaller than ${formatter.formatNumber(max)}`;
  }

  function ruleBigger(min: number) {
    return (value: number) =>
      value > min || `Value should bigger than ${formatter.formatNumber(min)}`;
  }

  return {
    ruleRequired,
    ruleBetween,
    ruleInteger,
    ruleDecimal,
    ruleSmaller,
    ruleBigger,
  };
}
