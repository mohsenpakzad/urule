import { ref } from 'vue';
import { defineStore } from 'pinia';
import { ProcessItem } from 'src/models';

export const useStore = defineStore('main', () => {

  const openedProcess = ref<ProcessItem>();

  return {
    openedProcess,
  }
});
