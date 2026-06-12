<script setup lang="ts">
import { ref, onMounted } from 'vue';
import NavBar from './components/layout/NavBar.vue';
import { versionApi } from './api';
import { loadConfig } from './composables/useConfig';
import { useAuth } from './composables/useAuth';
import { Toaster } from 'vue-sonner';
import 'vue-sonner/style.css';
import { CircleCheckBig } from 'lucide-vue-next';

const version = ref<string | null>(null);
// Gate the app on config: form defaults and dropdown options come from the
// server, so views must not render until the config has loaded.
const configReady = ref(false);
const configFailed = ref(false);

// Show the app chrome when authenticated, or always in open mode (auth off).
const { isAuthenticated, authEnabled } = useAuth();

onMounted(async () => {
  try {
    await loadConfig();
    configReady.value = true;
  } catch {
    configFailed.value = true;
  }

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
    <NavBar v-if="!authEnabled || isAuthenticated" />
    <main class="container mx-auto flex-1">
      <router-view v-if="configReady" />
      <div
        v-else-if="configFailed"
        class="flex h-64 items-center justify-center text-error"
      >
        Failed to load configuration. Is the backend reachable?
      </div>
      <div v-else class="flex h-64 items-center justify-center">
        <span class="loading loading-spinner loading-lg"></span>
      </div>
    </main>
    <footer class="py-4 text-center text-sm text-base-content/50 font-mono">
      <span v-if="version">v{{ version }}</span>
    </footer>
  </div>
</template>
