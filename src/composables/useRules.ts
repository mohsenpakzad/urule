export function useRules() {

  function ruleRequired(value: never) {
    return !!value || 'Required'
  }

  function ruleBetween(min: number, max: number) {
    return (value: number) =>
      value >= min && value <= max || `Value should between ${min.toLocaleString()} and ${max.toLocaleString()}`
  }

  function ruleInteger(value: string) {
    return /^[-+]?\d+$/.test(value) || 'Invalid integer'
  }

  function ruleDecimal(value: string) {
    return /^[+-]?((\d+(\.\d*)?)|(\.\d+))$/.test(value) || 'Invalid decimal'
  }

  function ruleSmaller(max: number) {
    return (value: number) =>
      value < max || `Value should smaller than ${max.toLocaleString()}`
  }

  function ruleBigger(min: number) {
    return (value: number) =>
      value > min || `Value should bigger than ${min.toLocaleString()}`
  }

  return {
    ruleRequired,
    ruleBetween,
    ruleInteger,
    ruleDecimal,
    ruleSmaller,
    ruleBigger,
  }
}
