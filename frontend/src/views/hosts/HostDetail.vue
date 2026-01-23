<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { hostsApi } from '@/api';
import type { HostWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import HostForm from '@/components/forms/HostForm.vue';

const route = useRoute();
const router = useRouter();

const host = ref<HostWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    host.value = await hostsApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load host';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await hostsApi.update(
      id,
      formData as Parameters<typeof hostsApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update host';
  }
}

async function handleDelete() {
  try {
    await hostsApi.delete(id);
    router.push('/hosts');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete host';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="host">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/hosts">Hosts</router-link></li>
              <li>{{ host.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ host.name }}
            <StatusBadge :status="host.status" />
          </h1>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-sm" @click="showEditModal = true">Edit</button>
          <button class="btn btn-sm btn-error" @click="showDeleteDialog = true">
            Delete
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">Details</h2>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <div class="text-sm text-base-content/70">Type</div>
                <div>{{ host.host_type }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Hostname</div>
                <div>{{ host.hostname || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">IP Address</div>
                <div>{{ host.ip_address || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Location</div>
                <div>{{ host.location || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">OS</div>
                <div>{{ host.os || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Specs</div>
                <div>{{ host.specs || '-' }}</div>
              </div>
            </div>
            <div v-if="host.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ host.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ host.applications.length }})
            </h2>
            <div
              v-if="host.applications.length === 0"
              class="text-base-content/70"
            >
              No applications linked
            </div>
            <div v-else class="overflow-x-auto">
              <table class="table table-sm">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Role</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="a in host.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
                    <td>
                      <span class="badge badge-outline">{{ a.role }}</span>
                    </td>
                    <td><StatusBadge :status="a.status" /></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>

    <Modal
      title="Edit Host"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <HostForm
        v-if="host"
        :host="host"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Host"
      message="Are you sure you want to delete this host?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
