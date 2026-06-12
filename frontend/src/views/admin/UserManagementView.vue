<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { toast } from 'vue-sonner';
import { usersApi } from '../../api';
import type { ResetRequestSummary, Role, UserSummary } from '../../types';

const roles: Role[] = ['admin', 'editor', 'viewer'];

const users = ref<UserSummary[]>([]);
const resetRequests = ref<ResetRequestSummary[]>([]);
const loading = ref(true);

// Create form
const newUsername = ref('');
const newEmail = ref('');
const newRole = ref<Role>('viewer');

// Setup-link modal
const linkModal = ref<HTMLDialogElement>();
const generatedUrl = ref('');

async function load() {
  loading.value = true;
  try {
    [users.value, resetRequests.value] = await Promise.all([
      usersApi.list(),
      usersApi.resetRequests(),
    ]);
  } catch (e: any) {
    toast.error(e.message || 'Failed to load users');
  } finally {
    loading.value = false;
  }
}

onMounted(load);

async function createUser() {
  try {
    await usersApi.create(
      newUsername.value,
      newEmail.value.trim() || null,
      newRole.value
    );
    toast.success('User created');
    newUsername.value = '';
    newEmail.value = '';
    newRole.value = 'viewer';
    await load();
  } catch (e: any) {
    toast.error(e.message || 'Could not create user');
  }
}

async function changeRole(u: UserSummary, role: Role) {
  try {
    await usersApi.updateRole(u.id, role);
    toast.success(`${u.username} is now ${role}`);
    await load();
  } catch (e: any) {
    toast.error(e.message || 'Could not change role');
    await load();
  }
}

async function removeUser(u: UserSummary) {
  if (!confirm(`Delete user "${u.username}"? This cannot be undone.`)) return;
  try {
    await usersApi.remove(u.id);
    toast.success('User deleted');
    await load();
  } catch (e: any) {
    toast.error(e.message || 'Could not delete user');
  }
}

async function generateLink(userId: string) {
  try {
    const { url } = await usersApi.setupLink(userId);
    generatedUrl.value = url;
    linkModal.value?.showModal();
    await load();
  } catch (e: any) {
    toast.error(e.message || 'Could not generate link');
  }
}

async function revoke(u: UserSummary) {
  try {
    await usersApi.revokeSessions(u.id);
    toast.success(`Revoked sessions for ${u.username}`);
  } catch (e: any) {
    toast.error(e.message || 'Could not revoke sessions');
  }
}

async function copyLink() {
  try {
    await navigator.clipboard.writeText(generatedUrl.value);
    toast.success('Copied to clipboard');
  } catch {
    // clipboard may be unavailable; the link is shown for manual copy
  }
}
</script>

<template>
  <div class="p-4 space-y-6">
    <h1 class="text-2xl font-bold">User management</h1>

    <!-- Pending reset requests -->
    <div
      v-if="resetRequests.length"
      class="alert alert-warning flex-col items-start"
    >
      <span class="font-semibold"
        >{{ resetRequests.length }} pending password reset request(s)</span
      >
      <ul class="w-full space-y-1">
        <li
          v-for="r in resetRequests"
          :key="r.id"
          class="flex items-center justify-between gap-2"
        >
          <span>{{ r.username }} ({{ r.created_at }})</span>
          <button class="btn btn-sm" @click="generateLink(r.user_id)">
            Generate reset link
          </button>
        </li>
      </ul>
    </div>

    <!-- Create user -->
    <div class="card bg-base-200">
      <div class="card-body">
        <h2 class="card-title text-lg">Create account</h2>
        <form
          @submit.prevent="createUser"
          class="flex flex-wrap items-end gap-3"
        >
          <input
            v-model="newUsername"
            required
            placeholder="Username"
            class="input input-bordered"
          />
          <input
            v-model="newEmail"
            type="email"
            placeholder="Email (optional)"
            class="input input-bordered"
          />
          <select v-model="newRole" class="select select-bordered">
            <option v-for="r in roles" :key="r" :value="r">{{ r }}</option>
          </select>
          <button type="submit" class="btn btn-primary">Create</button>
        </form>
        <p class="text-sm text-base-content/60">
          The user has no password until you generate a setup link for them.
        </p>
      </div>
    </div>

    <!-- Users table -->
    <div v-if="loading" class="flex justify-center py-8">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
    <div v-else class="overflow-x-auto">
      <table class="table">
        <thead>
          <tr>
            <th>Username</th>
            <th>Email</th>
            <th class="min-w-30">Role</th>
            <th>Password</th>
            <th class="text-center md:text-right min-w-30">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="u in users" :key="u.id">
            <td class="font-medium">{{ u.username }}</td>
            <td>{{ u.email ?? '—' }}</td>
            <td>
              <select
                class="select select-bordered select-sm"
                :value="u.role"
                @change="
                  changeRole(
                    u,
                    ($event.target as HTMLSelectElement).value as Role
                  )
                "
              >
                <option v-for="r in roles" :key="r" :value="r">{{ r }}</option>
              </select>
            </td>
            <td>
              <span
                class="badge badge-sm"
                :class="u.has_password ? 'badge-success' : 'badge-ghost'"
                >{{ u.has_password ? 'set' : 'not set' }}</span
              >
            </td>
            <td
              class="flex max-md:flex-col md:justify-end text-right gap-x-1 gap-y-1"
            >
              <button class="btn btn-sm" @click="generateLink(u.id)">
                Setup link
              </button>
              <button class="btn btn-sm" @click="revoke(u)">Revoke</button>
              <button class="btn btn-sm btn-error" @click="removeUser(u)">
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Setup link modal -->
    <dialog ref="linkModal" class="modal">
      <div class="modal-box">
        <h3 class="text-lg font-bold">Setup link</h3>
        <p class="mt-2 text-sm text-base-content/70">
          Share this one-time link with the user. It expires in 48 hours.
        </p>
        <div class="mt-3 flex gap-2">
          <input
            :value="generatedUrl"
            readonly
            class="input input-bordered flex-1 text-xs"
          />
          <button class="btn" @click="copyLink">Copy</button>
        </div>
        <div class="modal-action">
          <form method="dialog"><button class="btn">Close</button></form>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </div>
</template>
