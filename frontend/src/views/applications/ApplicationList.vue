<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { applicationsApi } from '@/api';
import type { Application, PaginatedResponse } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import Pagination from '@/components/common/Pagination.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';

const route = useRoute();
const router = useRouter();

const data = ref<PaginatedResponse<Application> | null>(null);
const loading = ref(true);
const error = ref('');
const showCreateModal = ref(false);
const search = ref((route.query.search as string) || '');

const page = ref(Number(route.query.page) || 1);

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    data.value = await applicationsApi.list({
      page: page.value,
      search: search.value || undefined,
    });
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to load applications';
  } finally {
    loading.value = false;
  }
}

async function handleCreate(formData: unknown) {
  try {
    await applicationsApi.create(
      formData as Parameters<typeof applicationsApi.create>[0]
    );
    showCreateModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to create application';
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
      <h1 class="text-2xl font-bold">Applications</h1>
      <button class="btn btn-primary" @click="showCreateModal = true">
        Add Application
      </button>
    </div>

    <div class="mb-4">
      <form @submit.prevent="handleSearch" class="join">
        <input
          v-model="search"
          type="text"
          placeholder="Search applications..."
          class="input input-bordered join-item w-64"
        />
        <button type="submit" class="btn join-item">Search</button>
      </form>
    </div>

    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <EmptyState
      v-else-if="!data?.data.length"
      message="No applications found"
      actionLabel="Add Application"
      @action="showCreateModal = true"
    />

    <div v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th>Name</th>
              <th>Description</th>
              <th>Status</th>
              <th>Repository</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="app in data.data"
              :key="app.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${app.id}`)"
            >
              <td class="font-medium">{{ app.name }}</td>
              <td class="max-w-md truncate">{{ app.description || '-' }}</td>
              <td><StatusBadge :status="app.status" /></td>
              <td>
                <a
                  v-if="app.repository_url"
                  :href="app.repository_url"
                  target="_blank"
                  class="link link-primary"
                  @click.stop
                  >Link</a
                >
                <span v-else>-</span>
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
      title="Create Application"
      :open="showCreateModal"
      @close="showCreateModal = false"
    >
      <ApplicationForm
        @submit="handleCreate"
        @cancel="showCreateModal = false"
      />
    </Modal>
  </div>
</template>
