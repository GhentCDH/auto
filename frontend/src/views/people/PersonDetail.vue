<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { peopleApi } from '@/api';
import type { PersonWithRelations } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import PersonForm from '@/components/forms/PersonForm.vue';

const route = useRoute();
const router = useRouter();

const person = ref<PersonWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    person.value = await peopleApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load person';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await peopleApi.update(
      id,
      formData as Parameters<typeof peopleApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update person';
  }
}

async function handleDelete() {
  try {
    await peopleApi.delete(id);
    router.push('/people');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete person';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="person">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li><router-link to="/people">People</router-link></li>
              <li>{{ person.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ person.name }}
            <span
              class="badge"
              :class="person.is_active ? 'badge-success' : 'badge-warning'"
              >{{ person.is_active ? 'Active' : 'Inactive' }}</span
            >
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
                <div class="text-sm text-base-content/70">Email</div>
                <div>{{ person.email || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Phone</div>
                <div>{{ person.phone || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Role</div>
                <div>{{ person.role || '-' }}</div>
              </div>
              <div>
                <div class="text-sm text-base-content/70">Department</div>
                <div>{{ person.department || '-' }}</div>
              </div>
            </div>
            <div v-if="person.notes" class="mt-4">
              <div class="text-sm text-base-content/70">Notes</div>
              <div>{{ person.notes }}</div>
            </div>
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              Applications ({{ person.applications.length }})
            </h2>
            <div
              v-if="person.applications.length === 0"
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
                    v-for="a in person.applications"
                    :key="a.id"
                    class="hover cursor-pointer"
                    @click="router.push(`/applications/${a.id}`)"
                  >
                    <td>{{ a.name }}</td>
                    <td>
                      <span class="badge badge-outline">{{
                        a.contribution_type
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
      title="Edit Person"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <PersonForm
        v-if="person"
        :person="person"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Person"
      message="Are you sure you want to delete this person?"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
