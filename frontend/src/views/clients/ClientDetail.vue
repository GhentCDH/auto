<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { clientsApi } from '@/api';
import type { ClientWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import ClientForm from '@/components/forms/ClientForm.vue';

const route = useRoute();
const router = useRouter();

const client = ref<ClientWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    client.value = await clientsApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load client';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await clientsApi.update(
      id,
      formData as Parameters<typeof clientsApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update client';
  }
}

async function handleDelete() {
  try {
    await clientsApi.delete(id);
    router.push('/clients');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete client';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="client">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/clients">Clients</router-link></li>
              <li>{{ client.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ client.name }}
            <StatusBadge :status="client.status" />
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
            <h2 class="card-title">Contact Information</h2>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <div class="text-sm text-base-content/70">Contact Name</div>
                <div>{{ client.contact_name || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Contact Email</div>
                <div>{{ client.contact_email || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Phone</div>
                <div>{{ client.phone || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Department</div>
                <div>{{ client.department || '-' }}</div>
              </div>
              <div class="col-span-2">
                <div class="text-sm text-base-content/70">Address</div>
                <div>{{ client.address || '-' }}</div>
              </div>
            </div>
            <div v-if="client.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ client.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ client.applications.length }})
            </h2>
            <div
              v-if="client.applications.length === 0"
              class="text-base-content/70"
            >
              No applications linked
            </div>
            <div v-else class="overflow-x-auto">
              <table class="table table-sm">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Relationship</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="a in client.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
                    <td>
                      <span class="badge badge-outline">{{
                        a.relationship_type
                      }}</span>
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
      title="Edit Client"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <ClientForm
        v-if="client"
        :client="client"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Client"
      message="Are you sure you want to delete this client?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
