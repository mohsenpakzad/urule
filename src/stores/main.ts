import { ref } from 'vue';
import { defineStore } from 'pinia';
import { Process } from 'src/models/core';

export const useStore = defineStore('main', () => {

  const openedProcess = ref<Process>();

  return {
    openedProcess,
  }
});
