<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { infraApi } from '@/api';
import type { InfraWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import { infraTypes } from '@/values';

const route = useRoute();
const router = useRouter();

const infra = ref<InfraWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    infra.value = await infraApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load infra';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await infraApi.update(
      id,
      formData as Parameters<typeof infraApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update infra';
  }
}

async function handleDelete() {
  try {
    await infraApi.delete(id);
    router.push('/infra');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete infra';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="infra">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li>
                <router-link to="/infra">Infrastructure</router-link>
              </li>
              <li>{{ infra.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ infra.name }}
            <span class="badge badge-outline">{{ infraTypes[infra.type as keyof typeof infraTypes] || infra.type }}</span>
          </h1>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-sm" @click="showEditModal = true">Edit</button>
          <button class="btn btn-sm btn-error" @click="showDeleteDialog = true">
            Delete
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div class="lg:col-span-2 space-y-6">
          <!-- Details Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Details</h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <div class="text-sm text-base-content/70">Description</div>
                  <div>{{ infra.description || '-' }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Type</div>
                  <div>{{ infraTypes[infra.type as keyof typeof infraTypes] || infra.type }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Created</div>
                  <div>{{ new Date(infra.created_at).toLocaleString() }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Updated</div>
                  <div>{{ new Date(infra.updated_at).toLocaleString() }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-6">
          <!-- Applications Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Applications</h2>
              <div
                v-if="infra.applications.length === 0"
                class="text-base-content/70"
              >
                No applications using this infrastructure
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="a in infra.applications"
                  :key="a.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/applications/${a.id}`"
                      class="link link-hover"
                    >
                      {{ a.name }}
                    </router-link>
                    <EnvironmentBadge
                      :environment="a.environment"
                      class="ml-2"
                    />
                  </div>
                  <StatusBadge :status="a.status" />
                </li>
              </ul>
            </div>
          </div>

          <!-- Services Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Services</h2>
              <div
                v-if="infra.services.length === 0"
                class="text-base-content/70"
              >
                No services using this infrastructure
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="s in infra.services"
                  :key="s.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/services/${s.id}`"
                      class="link link-hover"
                    >
                      {{ s.name }}
                    </router-link>
                    <EnvironmentBadge
                      :environment="s.environment"
                      class="ml-2"
                    />
                  </div>
                  <StatusBadge :status="s.status" />
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Infra Modal -->
    <Modal
      title="Edit Infra"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <InfraForm
        v-if="infra"
        :infra="infra"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Infrastructure"
      message="Are you sure you want to delete this infrastructure? This action cannot be undone."
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
