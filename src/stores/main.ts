import { computed, reactive, ref } from 'vue';
import { defineStore } from 'pinia';
import { QForm } from 'quasar';
import { useRules } from 'src/composables/useRules';
import { Location, Process } from 'src/models/core';
import { ScanState, ScanType, ScanValue, ValueType } from 'src/models/scan';

export const useStore = defineStore('main', () => {
  const rules = useRules();

  const scanTypes = [
    {
      label: 'Exact value',
      value: ScanType.Exact,
      availability: ScanState.FirstScan | ScanState.NextScan,
    },
    {
      label: 'Smaller than',
      value: ScanType.SmallerThan,
      availability: ScanState.FirstScan | ScanState.NextScan,
    },
    {
      label: 'Bigger than',
      value: ScanType.BiggerThan,
      availability: ScanState.FirstScan | ScanState.NextScan,
    },
    {
      label: 'Value between',
      value: ScanType.InRange,
      availability: ScanState.FirstScan | ScanState.NextScan,
    },
    {
      label: 'Unknown initial value',
      value: ScanType.Unknown,
      availability: ScanState.FirstScan,
    },
    {
      label: 'Increased value',
      value: ScanType.Increased,
      availability: ScanState.NextScan,
    },
    {
      label: 'Increased value by',
      value: ScanType.IncreasedBy,
      availability: ScanState.NextScan,
    },
    {
      label: 'Decreased value',
      value: ScanType.Decreased,
      availability: ScanState.NextScan,
    },
    {
      label: 'Decreased value by',
      value: ScanType.DecreasedBy,
      availability: ScanState.NextScan,
    },
    {
      label: 'Changed value',
      value: ScanType.Changed,
      availability: ScanState.NextScan,
    },
    {
      label: 'Unchanged value',
      value: ScanType.Unchanged,
      availability: ScanState.NextScan,
    },
  ];
  const valueTypes = [
    {
      label: 'I8 (Signed Byte)',
      value: ValueType.I8,
      min: (-2) ** 7,
      max: 2 ** 7 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'U8 (Unsigned Byte)',
      value: ValueType.U8,
      min: 0,
      max: 2 ** 8,
      format: rules.ruleInteger,
    },
    {
      label: 'I16 (Signed 2 Bytes)',
      min: (-2) ** 15,
      max: 2 ** 15 - 1,
      value: ValueType.I16,
      format: rules.ruleInteger,
    },
    {
      label: 'U16 (Unsigned 2 Bytes)',
      value: ValueType.U16,
      min: 0,
      max: 2 ** 16,
      format: rules.ruleInteger,
    },
    {
      label: 'I32 (Signed 4 Bytes)',
      value: ValueType.I32,
      min: (-2) ** 31,
      max: 2 ** 31 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'U32 (Unsigned 4 Bytes)',
      value: ValueType.U32,
      min: 0,
      max: 2 ** 32,
      format: rules.ruleInteger,
    },
    {
      label: 'I64 (Signed 8 Bytes)',
      value: ValueType.I64,
      min: (-2) ** 63,
      max: 2 ** 63 - 1,
      format: rules.ruleInteger,
    },
    {
      label: 'U64 (Unsigned 8 Bytes)',
      value: ValueType.U64,
      min: 0,
      max: 2 ** 64,
      format: rules.ruleInteger,
    },
    {
      label: 'F32 (Float 4 Bytes)',
      value: ValueType.F32,
      min: -3.40282347e38,
      max: 3.40282347e38,
      format: rules.ruleDecimal,
    },
    {
      label: 'F64 (Float 8 Bytes)',
      value: ValueType.F64,
      min: -1.7976931348623157e308,
      max: 1.7976931348623157e308,
      format: rules.ruleDecimal,
    },
  ];

  const openedProcess = ref<Process>();

  const scanState = ref<ScanState>(ScanState.FirstScan);
  const scanData = reactive({
    scanType: scanTypes.find((e) => e.value === ScanType.Exact),
    valueType: valueTypes.find((e) => e.value === ValueType.I32),
    value: <ScanValue>{
      Exact: '',
      Range: { start: '', end: '' },
    },
  });
  const scanForm = ref<QForm>();
  function resetToNextScan() {
    if (!(scanData.scanType!.availability & ScanState.NextScan)) {
      scanData.scanType = scanTypes.find((e) => e.value === ScanType.Exact);
    }
  }
  function resetToFirstScan() {
    if (!(scanData.scanType!.availability & ScanState.FirstScan)) {
      scanData.scanType = scanTypes.find((e) => e.value === ScanType.Exact);
    }
    scanData.value = <ScanValue>{
      Exact: '',
      Range: { start: '', end: '' },
    };
  }
  const locations = ref<Location[]>([]);
  const selectedLocations = ref<Location[]>([]);
  const locationsPagination = ref({
    rowsPerPage: 512,
    rowsNumber: 0,
    page: 1,
  });

  const scanTypeOptions = computed(() => {
    return scanTypes.filter((e) => e.availability & scanState.value);
  });
  const scanTypeOptionsRequiredInputs = computed(() => {
    switch (scanData.scanType?.value) {
      case ScanType.Exact:
      case ScanType.SmallerThan:
      case ScanType.BiggerThan:
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

    locations,
    selectedLocations,
    locationsPagination,

    //  computed
    scanTypeOptions,
    scanTypeOptionsRequiredInputs,

    // functions
    resetToNextScan,
    resetToFirstScan,
  };
});
