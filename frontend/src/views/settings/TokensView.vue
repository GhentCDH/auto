<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { toast } from 'vue-sonner';
import { Copy, Plus, Trash2, KeyRound } from 'lucide-vue-next';
import { authApi } from '@/api';
import type { ApiToken, CreatedApiToken } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';

const tokens = ref<ApiToken[]>([]);
const loading = ref(true);
const error = ref('');

// Create form
const name = ref('');
const expiresInDays = ref<number | null>(null);
const creating = ref(false);
const newToken = ref<CreatedApiToken | null>(null);

// Revoke confirm
const revoking = ref<ApiToken | null>(null);

function formatDate(value: string | null): string {
  if (!value) return 'never';
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

async function load() {
  loading.value = true;
  error.value = '';
  try {
    tokens.value = await authApi.listTokens();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load tokens';
  } finally {
    loading.value = false;
  }
}

async function create() {
  if (!name.value.trim()) return;
  creating.value = true;
  try {
    const days = Number(expiresInDays.value);
    newToken.value = await authApi.createToken({
      name: name.value.trim(),
      expires_in_days: Number.isFinite(days) && days > 0 ? days : undefined,
    });
    name.value = '';
    expiresInDays.value = null;
    toast.success('API token created');
    await load();
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to create token');
  } finally {
    creating.value = false;
  }
}

async function revoke() {
  if (!revoking.value) return;
  try {
    await authApi.deleteToken(revoking.value.id);
    toast.success('Token revoked');
    revoking.value = null;
    await load();
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to revoke token');
  }
}

async function copySecret() {
  if (!newToken.value) return;
  try {
    await navigator.clipboard.writeText(newToken.value.secret);
    toast.success('Token copied');
  } catch {
    toast.error('Could not copy token');
  }
}

onMounted(load);
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="mb-6">
      <h1 class="text-2xl font-bold flex items-center gap-3">
        <KeyRound class="w-6 h-6" /> API Tokens
      </h1>
      <p class="text-base-content/70 mt-1">
        Personal tokens for non-browser clients such as the TUI. They act as
        your account (current role), so treat them like passwords.
      </p>
    </div>

    <!-- New token reveal -->
    <div v-if="newToken" class="alert alert-success mb-6 items-start">
      <div class="w-full">
        <div class="font-semibold">Token "{{ newToken.name }}" created</div>
        <p class="text-sm mt-1">
          Copy it now. It is shown only once and cannot be retrieved again.
        </p>
        <div class="flex items-center gap-2 mt-3">
          <code
            class="font-mono text-xs break-all bg-base-100 rounded p-2 grow text-base-content"
            >{{ newToken.secret }}</code
          >
          <button class="btn btn-sm" @click="copySecret">
            <Copy class="w-4 h-4" /> Copy
          </button>
        </div>
      </div>
    </div>

    <!-- Create -->
    <div class="card bg-base-200 mb-6">
      <div class="card-body">
        <h2 class="card-title text-lg">Create token</h2>
        <form
          class="flex flex-col sm:flex-row gap-3 sm:items-end"
          @submit.prevent="create"
        >
          <fieldset class="fieldset grow">
            <legend class="fieldset-legend">Name *</legend>
            <input
              v-model="name"
              type="text"
              class="input w-full"
              placeholder="e.g., auto-tui laptop"
              required
            />
          </fieldset>
          <fieldset class="fieldset w-full sm:w-48">
            <legend class="fieldset-legend">Expires in (days)</legend>
            <input
              v-model.number="expiresInDays"
              type="number"
              min="1"
              class="input w-full"
              placeholder="never"
            />
          </fieldset>
          <button type="submit" class="btn btn-primary" :disabled="creating">
            <Plus class="w-4 h-4" /> Create
          </button>
        </form>
      </div>
    </div>

    <!-- List -->
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>
    <div v-else-if="tokens.length === 0" class="text-base-content/70">
      No API tokens yet.
    </div>
    <div v-else class="overflow-x-auto card bg-base-200">
      <table class="table">
        <thead>
          <tr>
            <th>Name</th>
            <th class="hidden md:table-cell">Created</th>
            <th class="hidden md:table-cell">Last used</th>
            <th>Expires</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="token in tokens" :key="token.id" class="hover">
            <td class="font-medium">{{ token.name }}</td>
            <td class="hidden md:table-cell">
              {{ formatDate(token.created_at) }}
            </td>
            <td class="hidden md:table-cell">
              {{ formatDate(token.last_used_at) }}
            </td>
            <td>
              {{ token.expires_at ? formatDate(token.expires_at) : 'never' }}
            </td>
            <td class="text-right">
              <button
                class="btn btn-ghost btn-xs text-error"
                title="Revoke"
                @click="revoking = token"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ConfirmDialog
      :open="revoking !== null"
      title="Revoke token"
      :message="`Revoke '${revoking?.name}'? Clients using it will lose access immediately.`"
      confirm-label="Revoke"
      danger
      @confirm="revoke"
      @cancel="revoking = null"
    />
  </div>
</template>
