import { computed, ref } from 'vue';
import { authApi } from '../api';
import { useConfig } from './useConfig';
import type { AuthUser } from '../types';

// Module-level singleton holding the current user. Populated once at app boot
// (see App.vue) and updated on login/logout.
const user = ref<AuthUser | null>(null);
let initPromise: Promise<void> | null = null;

/** Resolve the current session once; concurrent callers share the request. */
export function initAuth(): Promise<void> {
  if (!initPromise) {
    initPromise = authApi
      .me()
      .then((u) => {
        user.value = u;
      })
      .catch(() => {
        user.value = null;
      });
  }
  return initPromise;
}

export function useAuth() {
  const { config } = useConfig();

  // Auth is opt-in on the backend; when neither method is enabled the app runs
  // open and the frontend must not gate or restrict anything.
  const authEnabled = computed(() => {
    const a = config.value?.auth;
    return !!a && (a.password_enabled || a.oidc_enabled);
  });

  const isAuthenticated = computed(() => user.value !== null);
  const role = computed(() => user.value?.role ?? null);
  const isAdmin = computed(() => role.value === 'admin');
  // Admins and editors may mutate; viewers are read-only. In open mode everyone
  // can edit (there are no roles).
  const canEdit = computed(
    () => !authEnabled.value || role.value === 'admin' || role.value === 'editor'
  );

  async function login(username: string, password: string) {
    user.value = await authApi.login(username, password);
  }

  async function logout() {
    try {
      await authApi.logout();
    } finally {
      user.value = null;
    }
  }

  /** Refresh the cached user from the server (e.g. after an account link). */
  async function refresh() {
    user.value = await authApi.me().catch(() => null);
  }

  return {
    user: computed(() => user.value),
    authEnabled,
    isAuthenticated,
    role,
    isAdmin,
    canEdit,
    login,
    logout,
    refresh,
    initAuth,
  };
}
