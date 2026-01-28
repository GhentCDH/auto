<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { stacksApi } from '@/api';
import type { StackWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import StackForm from '@/components/forms/StackForm.vue';

const route = useRoute();
const router = useRouter();

const stack = ref<StackWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    stack.value = await stacksApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load stack';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await stacksApi.update(
      id,
      formData as Parameters<typeof stacksApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update stack';
  }
}

async function handleDelete() {
  try {
    await stacksApi.delete(id);
    router.push('/stack');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete stack';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="stack">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/stack">Technology Stack</router-link></li>
              <li>{{ stack.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ stack.name }}
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
            <div v-if="stack.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ stack.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ stack.applications.length }})
            </h2>
            <div
              v-if="stack.applications.length === 0"
              class="text-base-content/70"
            >
              No applications linked
            </div>
            <div v-else class="overflow-x-auto">
              <table class="table table-sm">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="a in stack.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
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
      title="Edit Technology"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <StackForm
        v-if="stack"
        :stack="stack"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Technology"
      message="Are you sure you want to delete this stack?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
