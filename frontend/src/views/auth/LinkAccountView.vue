<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { authApi } from '../../api';
import { useAuth } from '../../composables/useAuth';

const route = useRoute();
const router = useRouter();
const { refresh } = useAuth();
const token = route.query.token as string;

const username = ref('');
const password = ref('');
const submitting = ref(false);

async function submit() {
  submitting.value = true;
  try {
    await authApi.link(token, username.value, password.value);
    await refresh();
    toast.success('Account linked');
    router.push('/');
  } catch (e: any) {
    toast.error(e.message || 'Could not link account');
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="flex min-h-[70vh] items-center justify-center">
    <div class="card w-full max-w-sm bg-base-200 shadow-xl">
      <div class="card-body">
        <h1 class="card-title justify-center text-2xl wallefont">
          Link your account
        </h1>
        <p class="text-sm text-base-content/70">
          An account with your email already exists. Sign in with your existing
          password to link your SSO identity to it.
        </p>
        <form @submit.prevent="submit" class="space-y-3">
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
            Link and sign in
          </button>
        </form>
      </div>
    </div>
  </div>
</template>
