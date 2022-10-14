<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

onMounted(() => {
  setTimeout(async () => {
    await router.replace('/home');
  }, 3000);
});
</script>

<template>
  <q-page
    class="row items-center"
    style="height: 100vh; background-color: black"
  >
    <div class="col column items-center">
      <q-avatar square size="15rem">
        <img src="../../src-tauri/icons/icon.ico" />
      </q-avatar>
      <p class="q-mt-lg text-weight-bold text-h4" style="color: white">
        You are the one who rule😏
      </p>
    </div>
  </q-page>
</template>
