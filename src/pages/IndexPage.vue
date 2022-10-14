<script setup lang="ts">
import { computed, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useStore } from 'stores/main';
import { QForm, QTableColumn, useQuasar, ValidationRule } from 'quasar';
import { useUruleCore } from 'src/composables/useUruleCore';
import { useFormatter } from 'src/composables/useFormatter';
import { useRules } from 'src/composables/useRules';
import { Location } from 'src/models/core';
import { ScanState } from 'src/models/scan';

const store = useStore();
const uruleCore = useUruleCore();
const formatter = useFormatter();
const rules = useRules();
const q = useQuasar();

const {
  valueTypes: valueTypeOptions,

  scanData,

  resetUnknownScan,
  resetScanData,
} = store;

const {
  openedProcess,

  scanState,
  scanForm,
  scanTypeOptions,
  scanTypeOptionsRequiredInputs,

  locations,
  selectedLocations,
  locationsPagination,
} = storeToRefs(store);

const locationTableColumns = <QTableColumn[]>[
  {
    name: 'address',
    required: true,
    label: 'Address',
    align: 'left',
    sortable: true,
    field: (p: Location) => formatter.formatNumberToHex(p.address),
  },
  {
    name: 'value',
    required: true,
    label: 'Value',
    align: 'left',
    sortable: true,
    field: (p: Location) => p.value,
  },
];

const locationsLoading = ref(false);

const changeValueForm = ref<QForm>();
const changeValueDialog = ref<boolean>(false);
const changeValueDialogInput = ref<string>('');

function baseScanValueRules(): ValidationRule[] {
  if (scanData.valueType) {
    return [
      rules.ruleRequired,
      scanData.valueType.format,
      rules.ruleBetween(scanData.valueType.min, scanData.valueType.max),
    ];
  }
  return [];
}

const scanValueRules = computed(() => baseScanValueRules());
const scanValueMinRangeRules = computed((): ValidationRule[] => {
  if (scanData.value.Range) {
    const max = parseFloat(scanData.value.Range.end);
    const baseRules = baseScanValueRules();

    if (max) return [...baseRules, rules.ruleSmaller(max)];
    return baseRules;
  }
  return [];
});
const scanValueMaxRangeRules = computed((): ValidationRule[] => {
  if (scanData.value.Range) {
    const min = parseFloat(scanData.value.Range.start);
    const baseRules = baseScanValueRules();

    if (min) return [...baseRules, rules.ruleBigger(min)];
    return baseRules;
  }
  return [];
});

async function onFetchLocationsRequest({
  pagination,
}: {
  pagination: {
    page: number;
  };
}) {
  const { page } = pagination;
  locationsPagination.value.page = page;
  await fetchLocations();
}

async function fetchLocations() {
  locationsLoading.value = true;
  const { page, rowsPerPage } = locationsPagination.value;

  const [totalLocationsNumber, newLocations] = await uruleCore.getLastScan(
    locationsPagination.value.rowsPerPage,
    (page - 1) * rowsPerPage
  );
  locations.value.splice(0, locations.value.length, ...newLocations);
  locationsPagination.value.rowsNumber = totalLocationsNumber;

  locationsLoading.value = false;
}

async function firstScan() {
  if (!store.openedProcess || !(await scanForm.value?.validate())) return;
  q.loading.show();

  await uruleCore.firstScan(
    store.openedProcess.pid,
    scanData.valueType!.value,
    {
      typ: scanData.scanType!.value,
      value: scanData.value,
    }
  );
  await fetchLocations();

  scanState.value = ScanState.AfterInitialScan;
  resetUnknownScan();

  q.loading.hide();
}

async function nextScan() {
  if (!(await scanForm.value?.validate())) return;
  q.loading.show();

  await uruleCore.nextScan({
    typ: scanData.scanType!.value,
    value: scanData.value,
  });
  await fetchLocations();

  selectedLocations.value = [];
  q.loading.hide();
}

async function undoScan() {
  // todo
}

async function newScan() {
  await uruleCore.clearLastScan();

  resetScanData();
  locations.value = [];
  selectedLocations.value = [];
  await scanForm.value?.resetValidation();

  scanState.value = ScanState.BeforeInitialScan;
}

async function writeMemory() {
  if (!(await changeValueForm.value?.validate())) return;

  selectedLocations.value.forEach(async (location) => {
    const value = parseFloat(changeValueDialogInput.value);
    const writtenBytes = await uruleCore.writeOpenedProcessMemory(
      location.address,
      value
    );

    // TODO: instead of this, fetch these locations value from last scan again
    if (writtenBytes) {
      location.value = value;
    }
    console.log(`${writtenBytes} bytes written.`);
  });

  changeValueDialogInput.value = '';
  changeValueDialog.value = false;
}
</script>

<template>
  <q-page class="q-px-lg q-pt-lg" style="min-height: 0">
    <div class="q-gutter-y-md">
      <q-card class="q-px-lg q-pt-lg q-pb-sm" bordered flat>
        <q-form class="q-gutter-y-sm" ref="scanForm">
          <q-chip
            icon="saved_search"
            :removable="
              store.openedProcess && scanState === ScanState.BeforeInitialScan
            "
            @remove="store.openedProcess = undefined"
            size="medium"
            :ripple="false"
          >
            {{
              store.openedProcess
                ? formatter.formatProcess(store.openedProcess)
                : 'No Process Opened'
            }}
          </q-chip>

          <div class="row items-start q-mb-md">
            <template v-if="scanState === ScanState.BeforeInitialScan">
              <q-btn
                label="First Scan"
                icon="start"
                color="primary"
                @click.prevent="firstScan"
              >
                <q-menu v-if="!openedProcess" anchor="top right">
                  <q-banner class="bg-accent text-white" dense>
                    <template v-slot:avatar>
                      <q-icon name="playlist_add_check" />
                    </template>
                    You have to open a process from process list first!
                  </q-banner>
                </q-menu>
              </q-btn>
            </template>

            <template v-else-if="scanState === ScanState.AfterInitialScan">
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
          </div>

          <div class="row q-gutter-sm">
            <q-select
              class="col"
              label="Scan Type"
              :options="scanTypeOptions"
              outlined
              dense
              options-dense
              v-model="scanData.scanType"
            />
            <q-select
              class="col"
              label="Value Type"
              :options="valueTypeOptions"
              outlined
              dense
              options-dense
              v-model="scanData.valueType"
              :readonly="scanState === ScanState.AfterInitialScan"
            />
          </div>

          <div>
            <template v-if="scanTypeOptionsRequiredInputs === 1">
              <q-input
                label="Value"
                outlined
                dense
                v-model="scanData.value.Exact"
                reactive-rules
                :rules="scanValueRules"
                clearable
                :hint="
                  formatter.formatMinMaxValue(
                    scanData.valueType!.min,
                    scanData.valueType!.max
                  )
                "
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
                v-model="scanData.value.Range!.start"
                reactive-rules
                :rules="scanValueMinRangeRules"
                clearable
                :hint="
                  formatter.formatMinMaxValue(
                    scanData.valueType!.min,
                    scanData.valueType!.max
                  )
                "
              />

              <q-input
                class="col"
                label="Maximum Value"
                outlined
                dense
                v-model="scanData.value.Range!.end"
                reactive-rules
                :rules="scanValueMaxRangeRules"
                clearable
                :hint="
                  formatter.formatMinMaxValue(
                    scanData.valueType!.min,
                    scanData.valueType!.max
                  )
                "
              />
            </div>
          </div>
        </q-form>
      </q-card>

      <q-table
        class="q-pt-sm"
        style="height: 50vh"
        title="Found Locations"
        bordered
        flat
        dense
        :rows="locations"
        :columns="locationTableColumns"
        :rows-per-page-options="[0]"
        row-key="address"
        selection="multiple"
        v-model:selected="selectedLocations"
        v-model:pagination="locationsPagination"
        @request="onFetchLocationsRequest"
        :loading="locationsLoading"
        no-data-label="No locations found"
        :selected-rows-label="
          (numberOfRows) =>
            `${numberOfRows} location${numberOfRows > 1 ? 's' : ''} selected.`
        "
      >
        <template v-slot:loading>
          <q-inner-loading showing>
            <q-spinner-clock size="50" />
          </q-inner-loading>
        </template>
        <template v-slot:top-right>
          <q-btn
            v-if="selectedLocations.length > 0"
            label="Change value of selected addresses"
            color="primary"
            icon="edit"
            size="0.73rem"
            @click="changeValueDialog = true"
          />
          <q-dialog v-model="changeValueDialog">
            <q-card style="min-width: 350px">
              <q-card-section>
                <div class="text-h6">
                  Enter new value for selected addresses
                </div>
              </q-card-section>

              <q-card-section class="q-pt-none">
                <q-form ref="changeValueForm">
                  <q-input
                    dense
                    v-model="changeValueDialogInput"
                    autofocus
                    :rules="scanValueRules"
                    :hint="
                  formatter.formatMinMaxValue(
                    scanData.valueType!.min,
                    scanData.valueType!.max
                  )
                  "
                  />
                </q-form>
              </q-card-section>

              <q-card-actions align="right" class="text-primary">
                <q-btn
                  flat
                  label="Cancel"
                  v-close-popup
                  @click="changeValueDialogInput = ''"
                />
                <q-btn flat label="Save" @click="writeMemory" />
              </q-card-actions>
            </q-card>
          </q-dialog>
        </template>
      </q-table>
    </div>
  </q-page>
</template>
