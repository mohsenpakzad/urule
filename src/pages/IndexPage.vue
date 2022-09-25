<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useStore } from 'stores/main';
import { useUruleCore } from 'src/composables/useUruleCore';
import { useFormatter } from 'src/composables/useFormatter';
import { useRules } from 'src/composables/useRules';
import { Address } from 'src/models/core';
import { ScanState, ScanType, ValueType } from 'src/models/scan'

const store = useStore();
const uruleCore = useUruleCore();
const formatter = useFormatter();
const rules = useRules();


const scanState = ref<ScanState>(ScanState.BeforeInitialScan);

const addressTableColumns = [
  {
    name: 'name',
    required: true,
    label: 'Address',
    align: 'left',
    sortable: true,
    field: (p: Address) => p.name,
  },
  {
    name: 'value',
    required: true,
    label: 'Value',
    align: 'left',
    sortable: true,
    field: (p: Address) => p.value,
  },
];
const addressList = ref<Address[]>();

const allScanTypeOptions = [
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
const scanTypeOptions = computed(() => {
  return allScanTypeOptions.filter((e) => e.availability & scanState.value);
});

const valueTypeOptions = [
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

const scanForm = reactive({
  scanType: allScanTypeOptions.find(e => e.value === ScanType.Exact),
  valueType: valueTypeOptions.find(e => e.value === ValueType.I32),
  value: {
    exact: '',
    range: {min: '', max: ''}
  }
})

const scanTypeOptionsRequiredInputs = computed(() => {
  switch (scanForm.scanType?.value) {
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

function scanValueRules() {
  return [
    rules.ruleRequired,
    scanForm.valueType?.format,
    rules.ruleBetween(scanForm.valueType!.min, scanForm.valueType!.max),
  ]
}

const scanValueMinRangeRules = computed(() => {
  const max = parseFloat(scanForm.value.range.max);
  return [
    ...scanValueRules(),
    max ? rules.ruleSmaller(max) : undefined,
  ]
})
const scanValueMaxRangeRules = computed(() => {
  const min = parseFloat(scanForm.value.range.min);
  return [
    ...scanValueRules(),
    min ? rules.ruleBigger(min) : undefined,
  ]
})

async function firstScan() {
  if (!store.openedProcess) return;

  // todo

  scanState.value = ScanState.AfterInitialScan;
}

async function nextScan() {
  // todo
}

async function undoScan() {
  // todo
}

async function newScan() {
  addressList.value = []
  scanState.value = ScanState.BeforeInitialScan;
}
</script>

<template>
  <q-page class="q-px-lg q-pt-lg" style="min-height: 0">
    <div class="q-gutter-y-md">
      <q-card
        class="q-px-lg q-pt-lg q-pb-sm"
        bordered
        flat
      >
        <div class="q-gutter-y-sm">

          <q-chip
            icon="saved_search"
            :removable="store.openedProcess && scanState === ScanState.BeforeInitialScan"
            @remove="store.openedProcess = undefined"
            size="medium"
            :ripple="false"
          >
            {{
              store.openedProcess ? formatter.formatProcess(store.openedProcess) : 'No Process Opened'
            }}
          </q-chip>

          <q-form class="row items-start q-mb-md">
            <template
              v-if="scanState === ScanState.BeforeInitialScan"
            >
              <q-btn
                label="First Scan"
                icon="start"
                color="primary"
                @click.prevent="firstScan"
              >
                <q-menu
                  v-if="!store.openedProcess"
                  anchor="top right"
                >
                  <q-banner class="bg-accent text-white" dense>
                    <template v-slot:avatar>
                      <q-icon name="playlist_add_check"/>
                    </template>
                    You have to open a process from process list first!
                  </q-banner>
                </q-menu>
              </q-btn>
            </template>

            <template
              v-else-if="scanState === ScanState.AfterInitialScan"
            >
              <div class="q-gutter-sm">
                <q-btn
                  label="Next Scan"
                  icon="navigate_next"
                  color="positive"
                  @click="nextScan"
                />

                <q-btn
                  label="Undo Scan"
                  icon="undo"
                  color="negative"
                  @click="undoScan"
                />
              </div>

              <q-space></q-space>

              <q-btn
                label="New Scan"
                icon="keyboard_tab"
                color="primary"
                @click="newScan"
                type="reset"
              />
            </template>
          </q-form>

          <div class="row q-gutter-sm">
            <q-select
              class="col"
              label="Scan Type"
              :options="scanTypeOptions"
              outlined
              dense
              options-dense
              v-model="scanForm.scanType"
            />
            <q-select
              class="col"
              label="Value Type"
              :options="valueTypeOptions"
              outlined
              dense
              options-dense
              v-model="scanForm.valueType"
              :readonly="scanState === ScanState.AfterInitialScan"
            />
          </div>

          <div>
            <template
              v-if="scanTypeOptionsRequiredInputs === 1"
            >
              <q-input
                label="Value"
                outlined
                dense
                v-model="scanForm.value.exact"
                :rules="scanValueRules()"
                clearable
              />
            </template>
            <div
              class="row q-gutter-x-sm"
              v-else-if="scanTypeOptionsRequiredInputs === 2"
            >
              <q-input
                class="col"
                label="Minimum Value"
                outlined
                dense
                v-model="scanForm.value.range.min"
                reactive-rules
                :rules="scanValueMinRangeRules"
                clearable
              />

              <q-input
                class="col"
                label="Maximum Value"
                outlined
                dense
                v-model="scanForm.value.range.max"
                reactive-rules
                :rules="scanValueMaxRangeRules"
                clearable
              />
            </div>
          </div>

        </div>
      </q-card>

      <q-table
        class="q-pa-lg"
        style="height: 55vh"
        bordered
        flat
        dense
        :rows="addressList"
        :columns="addressTableColumns"
        rows-per-page-options="0"
        row-key="name"
      >
        <template v-slot:body="props">
          <q-tr :props="props">
            <q-td key="name" :props="props">{{ props.row.name }}</q-td>
            <q-td key="value" :props="props">
              {{ props.row.value }}
              <q-popup-edit
                :ref="addressPopup"
                v-model="props.row.value"
                title="Update value"
              >
                <q-input
                  type="number"
                  v-model="props.row.value"
                  dense
                  autofocus
                  hint="Use buttons to close"
                />
                <div class="row items-center">
                  <q-btn
                    class="col"
                    label="Set"
                    flat
                    @click.stop.prevent="writeMemory(props.row)"
                  />
                </div>
              </q-popup-edit>
            </q-td>
          </q-tr>
        </template>
      </q-table>
    </div>
  </q-page>
</template>
