<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api';
import { ProcessItem } from 'src/models';
import { onStartTyping } from '@vueuse/core';
import { useStore } from 'stores/main';

const store = useStore();
const router = useRouter();

const processListColumns = [
  {
    name: 'name',
    required: true,
    label: 'Name',
    align: 'left',
    sortable: true,
    field: (p: ProcessItem) => p.name,
  },
  {
    name: 'pid',
    required: true,
    label: 'Pid',
    align: 'left',
    sortable: true,
    field: (p: ProcessItem) => p.pid,
  },
];
const processList = ref<ProcessItem[]>([])
const processesFilter = ref<string>()
const selectedProcess = ref([])

const searchInput = ref<HTMLInputElement | null>(null)

async function openProcess() {
  if (selectedProcess.value.length < 1) return
  store.openedProcess = selectedProcess.value[0]
  router.back()
}

onMounted(async () => {
  // create interval function
  processList.value = await invoke<ProcessItem[]>('get_processes');
  setInterval(async () => processList.value = await invoke<ProcessItem[]>('get_processes'), 1000);
});

onStartTyping(() => {
    searchInput.value?.focus()
})
</script>

<template>
  <q-page>
    <q-table
      class="table"
      title="Processes"
      dense
      :rows="processList"
      :columns="processListColumns"
      rows-per-page-options="0"
      :filter="processesFilter"
      row-key="pid"
      selection="single"
      v-model:selected="selectedProcess"
    >
        <template v-slot:top-right>
          <q-input
            ref="searchInput"
            borderless
            dense
            debounce="300"
            v-model="processesFilter"
            placeholder="Type To Search"
            style="caret-color: transparent;"
          >
            <template v-slot:append>
              <q-icon name="search" />
            </template>
          </q-input>
        </template>
    </q-table>
    <div class="row justify-evenly items-center sub-table">
        <q-btn
          style="width: 10rem"
          color="primary"
          @click="openProcess"
        >
          Open
          <q-popup-proxy v-if="selectedProcess.length < 1">
            <q-banner class="bg-primary text-white" dense>
              <template v-slot:avatar>
                <q-icon name="done"/>
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
  </q-page>
</template>

<style scoped>
.table {
  height: 90vh;
}
.sub-table {
  height: 10vh;
}
</style>
