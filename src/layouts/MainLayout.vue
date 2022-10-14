<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const toolbarButtons = computed(() => {
  switch (router.currentRoute.value.path) {
    case '/':
      return [
        {
          label: 'Process list',
          icon: 'apps',
          click: async () => await router.push('/process-list'),
        },
      ];
    default:
      return [
        {
          label: 'Back',
          icon: 'keyboard_return',
          click: async () => await router.back(),
        },
      ];
  }
});
</script>

<template>
  <q-layout view="hHh lpR fFf">
    <q-header class="bg-primary text-secondary q-py-sm">
      <q-toolbar>
        <q-btn
          v-for="(button, i) in toolbarButtons"
          :key="i"
          :label="button.label"
          :icon="button.icon"
          @click="button.click"
          rounded
          color="secondary"
          text-color="primary"
        ></q-btn>

        <q-space />

        <q-avatar square>
          <img src="../../src-tauri/icons/icon.ico" />
        </q-avatar>

        <q-toolbar-title shrink>Urule</q-toolbar-title>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>
