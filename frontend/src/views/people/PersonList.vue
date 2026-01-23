<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { peopleApi } from '@/api';
import type { Person, PaginatedResponse } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import Pagination from '@/components/common/Pagination.vue';
import Modal from '@/components/common/Modal.vue';
import PersonForm from '@/components/forms/PersonForm.vue';

const route = useRoute();
const router = useRouter();

const data = ref<PaginatedResponse<Person> | null>(null);
const loading = ref(true);
const error = ref('');
const showCreateModal = ref(false);
const search = ref((route.query.search as string) || '');
const page = ref(Number(route.query.page) || 1);

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    data.value = await peopleApi.list({
      page: page.value,
      search: search.value || undefined,
    });
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load people';
  } finally {
    loading.value = false;
  }
}

async function handleCreate(formData: unknown) {
  try {
    await peopleApi.create(formData as Parameters<typeof peopleApi.create>[0]);
    showCreateModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create person';
  }
}

function handlePageChange(newPage: number) {
  page.value = newPage;
  router.push({ query: { ...route.query, page: newPage } });
}

function handleSearch() {
  page.value = 1;
  router.push({
    query: { ...route.query, search: search.value || undefined, page: 1 },
  });
}

watch(() => route.query, loadData, { immediate: true });
onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <div
      class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
    >
      <h1 class="text-2xl font-bold">People</h1>
      <button class="btn btn-primary" @click="showCreateModal = true">
        Add Person
      </button>
    </div>

    <div class="mb-4">
      <form @submit.prevent="handleSearch" class="join">
        <input
          v-model="search"
          type="text"
          placeholder="Search people..."
          class="input input-bordered join-item w-64"
        />
        <button type="submit" class="btn join-item">Search</button>
      </form>
    </div>

    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>
    <EmptyState
      v-else-if="!data?.data.length"
      message="No people found"
      actionLabel="Add Person"
      @action="showCreateModal = true"
    />

    <div v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th>Name</th>
              <th>Email</th>
              <th>Role</th>
              <th>Department</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="person in data.data"
              :key="person.id"
              class="hover cursor-pointer"
              @click="router.push(`/people/${person.id}`)"
            >
              <td class="font-medium">{{ person.name }}</td>
              <td>{{ person.email || '-' }}</td>
              <td>{{ person.role || '-' }}</td>
              <td>{{ person.department || '-' }}</td>
              <td>
                <span
                  class="badge"
                  :class="person.is_active ? 'badge-success' : 'badge-warning'"
                  >{{ person.is_active ? 'Active' : 'Inactive' }}</span
                >
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <Pagination
        :page="data.page"
        :total-pages="data.total_pages"
        :total="data.total"
        @update:page="handlePageChange"
      />
    </div>

    <Modal
      title="Create Person"
      :open="showCreateModal"
      @close="showCreateModal = false"
    >
      <PersonForm @submit="handleCreate" @cancel="showCreateModal = false" />
    </Modal>
  </div>
</template>
