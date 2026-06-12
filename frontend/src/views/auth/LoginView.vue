<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { useAuth } from '../../composables/useAuth';
import { useConfig } from '../../composables/useConfig';
import { authApi } from '../../api';

const router = useRouter();
const { login } = useAuth();
const { config } = useConfig();

const passwordEnabled = computed(() => config.value?.auth.password_enabled ?? true);
const oidcEnabled = computed(() => config.value?.auth.oidc_enabled ?? false);

const username = ref('');
const password = ref('');
const submitting = ref(false);

async function submit() {
  submitting.value = true;
  try {
    await login(username.value, password.value);
    router.push('/');
  } catch (e: any) {
    toast.error(e.message || 'Login failed');
  } finally {
    submitting.value = false;
  }
}

function loginWithOidc() {
  window.location.assign(authApi.oidcStartUrl);
}

// Forgot-password (logged-out reset request)
const showReset = ref(false);
const resetUsername = ref('');
async function submitReset() {
  try {
    await authApi.resetRequest(resetUsername.value);
    toast.success(
      'If that account exists, an admin has been notified to issue a reset link.'
    );
    showReset.value = false;
    resetUsername.value = '';
  } catch (e: any) {
    toast.error(e.message || 'Could not submit request');
  }
}
</script>

<template>
  <div class="flex min-h-[70vh] items-center justify-center">
    <div class="card w-full max-w-sm bg-base-200 shadow-xl">
      <div class="card-body">
        <h1 class="card-title justify-center text-2xl wallefont">Sign in</h1>

        <form
          v-if="passwordEnabled && !showReset"
          @submit.prevent="submit"
          class="space-y-3"
        >
          <input
            v-model="username"
            type="text"
            required
            placeholder="Username"
            autocomplete="username"
            class="input input-bordered w-full"
          />
          <input
            v-model="password"
            type="password"
            required
            placeholder="Password"
            autocomplete="current-password"
            class="input input-bordered w-full"
          />
          <button
            type="submit"
            class="btn btn-primary w-full"
            :disabled="submitting"
          >
            Sign in
          </button>
          <button
            type="button"
            class="btn btn-link btn-sm w-full"
            @click="showReset = true"
          >
            Forgot password?
          </button>
        </form>

        <form
          v-else-if="showReset"
          @submit.prevent="submitReset"
          class="space-y-3"
        >
          <p class="text-sm text-base-content/70">
            Enter your username. An admin will be notified and can issue you a
            reset link.
          </p>
          <input
            v-model="resetUsername"
            type="text"
            required
            placeholder="Username"
            class="input input-bordered w-full"
          />
          <button type="submit" class="btn btn-primary w-full">
            Request reset
          </button>
          <button
            type="button"
            class="btn btn-link btn-sm w-full"
            @click="showReset = false"
          >
            Back to sign in
          </button>
        </form>

        <div v-if="oidcEnabled && !showReset">
          <div v-if="passwordEnabled" class="divider text-xs">OR</div>
          <button class="btn btn-outline w-full" @click="loginWithOidc">
            Sign in with SSO
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
