<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { authApi } from '../../api';

const route = useRoute();
const router = useRouter();
const token = route.params.token as string;

const password = ref('');
const confirm = ref('');
const submitting = ref(false);

async function submit() {
  if (password.value !== confirm.value) {
    toast.error('Passwords do not match');
    return;
  }
  submitting.value = true;
  try {
    await authApi.setPassword(token, password.value);
    toast.success('Password set. You can now sign in.');
    router.push('/login');
  } catch (e: any) {
    toast.error(e.message || 'Could not set password');
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
          Set your password
        </h1>
        <form @submit.prevent="submit" class="space-y-3">
          <input
            v-model="password"
            type="password"
            required
            minlength="8"
            placeholder="New password"
            autocomplete="new-password"
            class="input input-bordered w-full"
          />
          <input
            v-model="confirm"
            type="password"
            required
            minlength="8"
            placeholder="Confirm password"
            autocomplete="new-password"
            class="input input-bordered w-full"
          />
          <button
            type="submit"
            class="btn btn-primary w-full"
            :disabled="submitting"
          >
            Set password
          </button>
        </form>
      </div>
    </div>
  </div>
</template>
