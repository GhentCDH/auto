import { ref } from 'vue';
import { configApi } from '../api';
import type { PublicConfig } from '../types';

// Module-level singleton: the server is the single source of truth for form
// defaults and dropdown options. Loaded once at app boot (see App.vue) before
// any config-dependent view renders, so consumers can treat it as present.
const config = ref<PublicConfig | null>(null);
let loadPromise: Promise<PublicConfig> | null = null;

/** Fetch the config once; concurrent callers share the same request. */
export function loadConfig(): Promise<PublicConfig> {
  if (!loadPromise) {
    loadPromise = configApi.get().then((c) => {
      config.value = c;
      return c;
    });
  }
  return loadPromise;
}

/**
 * Access the loaded config. Throws if called before {@link loadConfig}
 * resolves — callers render only after the app gates on config readiness.
 */
export function requireConfig(): PublicConfig {
  if (!config.value) {
    throw new Error('config accessed before it was loaded');
  }
  return config.value;
}

export function useConfig() {
  return { config, loadConfig, requireConfig };
}
