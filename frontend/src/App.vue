<script setup lang="ts">
import { ref, onMounted } from 'vue';
import NavBar from './components/layout/NavBar.vue';
import { versionApi } from './api';
import { Toaster } from 'vue-sonner';
import 'vue-sonner/style.css';
import { CircleCheckBig } from 'lucide-vue-next';

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
  <Toaster
    position="bottom-left"
    :visibleToasts="5"
    :toastOptions="{
      unstyled: false,
      classes: {
        toast: 'rounded-box!',
        error: 'bg-error!',
        success: '',
        warning: 'bg-warning!',
        info: 'bg-base-200!',
      },
    }"
  >
    <template #success-icon>
      <CircleCheckBig class="text-success" :size="18" />
    </template>
  </Toaster>
  <div class="flex min-h-screen flex-col bg-base-100">
    <NavBar />
    <main class="container mx-auto flex-1">
      <router-view />
    </main>
    <footer class="py-4 text-center text-sm text-base-content/50 font-mono">
      <span v-if="version">v{{ version }}</span>
    </footer>
  </div>
</template>
