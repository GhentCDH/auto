<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { domainsApi } from '@/api';
import type { DomainWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import DomainForm from '@/components/forms/DomainForm.vue';

const route = useRoute();
const router = useRouter();

const domain = ref<DomainWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    domain.value = await domainsApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load domain';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await domainsApi.update(
      id,
      formData as Parameters<typeof domainsApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update domain';
  }
}

async function handleDelete() {
  try {
    await domainsApi.delete(id);
    router.push('/domains');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete domain';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="domain">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/domains">Domains</router-link></li>
              <li>{{ domain.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ domain.name }}
            <StatusBadge :status="domain.status" />
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
                <div class="text-sm text-base-content/70">Registrar</div>
                <div>{{ domain.registrar || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">DNS Provider</div>
                <div>{{ domain.dns_provider || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Expires</div>
                <div>{{ domain.expires_at || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">SSL Expires</div>
                <div>{{ domain.ssl_expires_at || '-' }}</div>
              </div>
              <div class="col-span-2">
                <div class="text-sm text-base-content/70">SSL Issuer</div>
                <div>{{ domain.ssl_issuer || '-' }}</div>
              </div>
            </div>
            <div v-if="domain.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ domain.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ domain.applications.length }})
            </h2>
            <div
              v-if="domain.applications.length === 0"
              class="text-base-content/70"
            >
              No applications linked
            </div>
            <div v-else class="overflow-x-auto">
              <table class="table table-sm">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Record Type</th>
                    <th>Primary</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="a in domain.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
                    <td>{{ a.record_type }}</td>
                    <td>{{ a.is_primary ? 'Yes' : 'No' }}</td>
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
      title="Edit Domain"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <DomainForm
        v-if="domain"
        :domain="domain"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Domain"
      message="Are you sure you want to delete this domain?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
