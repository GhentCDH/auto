<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { sharesApi } from '@/api';
import type { NetworkShareWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import ShareForm from '@/components/forms/ShareForm.vue';

const route = useRoute();
const router = useRouter();

const share = ref<NetworkShareWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    share.value = await sharesApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load share';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await sharesApi.update(
      id,
      formData as Parameters<typeof sharesApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update share';
  }
}

async function handleDelete() {
  try {
    await sharesApi.delete(id);
    router.push('/shares');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete share';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="share">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/shares">Network Shares</router-link></li>
              <li>{{ share.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ share.name }}
            <StatusBadge :status="share.status" />
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
                <div>{{ share.share_type.toUpperCase() }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Server</div>
                <div>{{ share.server || '-' }}</div>
              </div>
              <div class="col-span-2">
                <div class="text-sm text-base-content/70">Path</div>
                <div class="font-mono">{{ share.path }}</div>
              </div>
              <div class="col-span-2">
                <div class="text-sm text-base-content/70">Purpose</div>
                <div>{{ share.purpose || '-' }}</div>
              </div>
            </div>
            <div v-if="share.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ share.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ share.applications.length }})
            </h2>
            <div
              v-if="share.applications.length === 0"
              class="text-base-content/70"
            >
              No applications linked
            </div>
            <div v-else class="overflow-x-auto">
              <table class="table table-sm">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Usage</th>
                    <th>Mount Point</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="a in share.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
                    <td>{{ a.usage || '-' }}</td>
                    <td class="font-mono text-xs">
                      {{ a.mount_point || '-' }}
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
      title="Edit Network Share"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <ShareForm
        v-if="share"
        :share="share"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Network Share"
      message="Are you sure you want to delete this share?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
