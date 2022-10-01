import { computed, reactive, ref } from 'vue';
import { defineStore } from 'pinia';
import { QForm } from 'quasar';
import { useRules } from 'src/composables/useRules';
import { Address, Process } from 'src/models/core';
import { ScanState, ScanType, ValueType } from 'src/models/scan';

export const useStore = defineStore('main', () => {

  const rules = useRules();

  const scanTypes = [
    {
      label: 'Exact value',
      value: ScanType.Exact,
      availability: ScanState.BeforeInitialScan | ScanState.AfterInitialScan,
    },
    {
      label: 'Value between',
      value: ScanType.InRange,
      availability: ScanState.BeforeInitialScan | ScanState.AfterInitialScan,
    },
    {
      label: 'Unknown initial value',
      value: ScanType.Unknown,
      availability: ScanState.BeforeInitialScan,
    },
    {
      label: 'Increased value',
      value: ScanType.Increased,
      availability: ScanState.AfterInitialScan,
    },
    {
      label: 'Increased value by',
      value: ScanType.IncreasedBy,
      availability: ScanState.AfterInitialScan,
    },
    {
      label: 'Decreased value',
      value: ScanType.Decreased,
      availability: ScanState.AfterInitialScan,
    },
    {
      label: 'Decreased value by',
      value: ScanType.DecreasedBy,
      availability: ScanState.AfterInitialScan,
    },
    {
      label: 'Changed value',
      value: ScanType.Changed,
      availability: ScanState.AfterInitialScan,
    },
    {
      label: 'Unchanged value',
      value: ScanType.Unchanged,
      availability: ScanState.AfterInitialScan,
    },
  ];
  const valueTypes = [
    {
      label: 'Signed Byte',
      value: ValueType.I8,
      min: (-2) ** 7,
      max: 2 ** 7 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'Unsigned Byte',
      value: ValueType.U8,
      min: 0,
      max: 2 ** 8,
      format: rules.ruleInteger,
    },
    {
      label: 'Signed 2 Bytes',
      min: (-2) ** 15,
      max: 2 ** 15 - 1,
      value: ValueType.I16,
      format: rules.ruleInteger,
    },
    {
      label: 'Unsigned 2 Bytes',
      value: ValueType.U16,
      min: 0,
      max: 2 ** 16,
      format: rules.ruleInteger,
    },
    {
      label: 'Signed 4 Bytes',
      value: ValueType.I32,
      min: (-2) ** 31,
      max: 2 ** 31 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'Unsigned 4 Bytes',
      value: ValueType.U32,
      min: 0,
      max: 2 ** 32,
      format: rules.ruleInteger,
    },
    {
      label: 'Signed 8 Bytes',
      value: ValueType.I64,
      min: (-2) ** 63,
      max: 2 ** 63 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'Unsigned 8 Bytes',
      value: ValueType.U64,
      min: 0,
      max: 2 ** 64,
      format: rules.ruleInteger,
    },
    {
      label: 'Float 4 Bytes',
      value: ValueType.F32,
      min: -3.40282347e+38,
      max: 3.40282347e+38,
      format: rules.ruleDecimal,
    },
    {
      label: 'Float 8 Bytes',
      value: ValueType.F64,
      min: -1.7976931348623157e+308,
      max: 1.7976931348623157e+308,
      format: rules.ruleDecimal,
    },
  ];

  const openedProcess = ref<Process>();

  const scanState = ref<ScanState>(ScanState.BeforeInitialScan);
  const scanData = reactive({
    scanType: scanTypes.find(e => e.value === ScanType.Exact),
    valueType: valueTypes.find(e => e.value === ValueType.I32),
    value: {
      exact: '',
      range: {min: '', max: ''}
    }
  });
  const scanForm = ref<QForm>();

  const addressList = ref<Address[]>([]);
  const selectedAddresses = ref<Address[]>([]);


  const scanTypeOptions = computed(() => {
    return scanTypes.filter((e) => e.availability & scanState.value);
  });
  const scanTypeOptionsRequiredInputs = computed(() => {
    switch (scanData.scanType?.value) {
      case ScanType.Exact:
      case ScanType.DecreasedBy:
      case ScanType.IncreasedBy:
        return 1;
      case ScanType.InRange:
        return 2;
      default:
        return 0;
    }
  });

  return {
    // constants
    scanTypes,
    valueTypes,

    // refs and reactives
    openedProcess,

    scanState,
    scanData,
    scanForm,

    addressList,
    selectedAddresses,

    //  computed
    scanTypeOptions,
    scanTypeOptionsRequiredInputs,
  }
});
