<script setup lang="ts">
import { ref, onMounted } from 'vue';
import NavBar from './components/layout/NavBar.vue';
import { versionApi } from './api';

const version = ref<string | null>(null);

onMounted(async () => {
  try {
    const data = await versionApi.get();
    version.value = data.version;
  } catch {
    // Silently fail - version display is not critical
  }
});
</script>

<template>
  <div class="flex min-h-screen flex-col bg-base-100">
    <NavBar />
    <main class="container mx-auto max-w-7xl flex-1">
      <router-view />
    </main>
    <footer class="py-4 text-center text-sm text-base-content/50 font-mono">
      <span v-if="version">v{{ version }}</span>
    </footer>
  </div>
</template>
