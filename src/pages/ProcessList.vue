<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useStore } from 'stores/main';
import { QTableColumn } from 'quasar';
import { onStartTyping, useIntervalFn } from '@vueuse/core';
import { useUruleCore } from 'src/composables/useUruleCore';
import { useFormatter } from 'src/composables/useFormatter';
import { Process } from 'src/models/core';

const store = useStore();
const router = useRouter();
const uruleCore = useUruleCore();
const formatter = useFormatter();

const processListColumns = <QTableColumn[]>[
  {
    name: 'name',
    required: true,
    label: 'Name',
    align: 'left',
    sortable: true,
    field: (p: Process) => p.name,
  },
  {
    name: 'pid',
    required: true,
    label: 'Pid',
    align: 'left',
    sortable: true,
    field: (p: Process) => p.pid,
  },
];
const processList = ref<Process[]>([]);
const processesFilter = ref<string>();
const selectedProcess = ref<Process[]>([]);

const searchInput = ref<HTMLInputElement | null>(null);

const processListLoading = ref<boolean>(true);

function getSelectedProcessLabel() {
  return `${formatter.formatProcess(selectedProcess.value[0])} selected.`;
}
// Single selection logic
function onProcessSelection({} = {}, row: Process) {
  if (selectedProcess.value.includes(row)) {
    selectedProcess.value = [];
  } else {
    selectedProcess.value = [row];
  }
}

async function openProcess() {
  if (selectedProcess.value.length < 1) return;

  await uruleCore.clearScanData();
  store.resetToFirstScan();

  store.openedProcess = selectedProcess.value[0];

  router.back();
}

onStartTyping(() => {
  searchInput.value?.focus();
});

onMounted(async () => {
  useIntervalFn(
    async () => {
      processList.value = (await uruleCore.getProcesses()).reverse();
      if (processListLoading.value) processListLoading.value = false;
    },
    1000,
    { immediateCallback: true }
  );
});
</script>

<template>
  <q-page class="q-px-lg q-pt-lg" style="min-height: 0">
    <q-table
      class="my-sticky-header-table"
      style="height: 70vh"
      title="Processes"
      flat
      bordered
      dense
      :rows="processList"
      :columns="processListColumns"
      :rows-per-page-options="[0]"
      :filter="processesFilter"
      row-key="pid"
      selection="single"
      v-model:selected="selectedProcess"
      :selected-rows-label="getSelectedProcessLabel"
      :loading="processListLoading"
      @row-click="onProcessSelection"
    >
      <template v-slot:top-right>
        <q-input
          ref="searchInput"
          borderless
          dense
          debounce="300"
          v-model="processesFilter"
          placeholder="Type To Search"
          style="caret-color: transparent"
        >
          <template v-slot:append>
            <q-icon name="search" />
          </template>
        </q-input>
      </template>
    </q-table>

    <div class="row justify-evenly items-center" style="height: 15vh">
      <q-btn style="width: 10rem" color="primary" @click="openProcess">
        Open
        <q-popup-proxy v-if="selectedProcess.length < 1">
          <q-banner class="bg-accent text-white" dense>
            <template v-slot:avatar>
              <q-icon name="done" />
            </template>
            Please select a process first!
          </q-banner>
        </q-popup-proxy>
      </q-btn>
      <q-btn
        style="width: 10rem"
        color="primary"
        @click="router.back()"
        outline
      >
        Cancel
      </q-btn>
    </div>
    <p class="text-weight-medium">
      <q-icon name="admin_panel_settings" />
      If you can't see your desired process in this list, try to run app as
      administrator.
    </p>
  </q-page>
</template>

<style lang="sass">
.my-sticky-header-table
  /* height or max-height is important */
  height: 310px

  .q-table__top,
  .q-table__bottom,
  thead tr:first-child th
    /* bg color is important for th; just specify one */
    background-color: white

  thead tr th
    position: sticky
    z-index: 1

  thead tr:first-child th
    top: 0

  /* this is when the loading indicator appears */

  &.q-table--loading thead tr:last-child th
    /* height of all previous header rows */
    top: 48px
</style>
