<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { hostsApi } from '@/api';
import type { Host, PaginatedResponse } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import Pagination from '@/components/common/Pagination.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import HostForm from '@/components/forms/HostForm.vue';

const route = useRoute();
const router = useRouter();

const data = ref<PaginatedResponse<Host> | null>(null);
const loading = ref(true);
const error = ref('');
const showCreateModal = ref(false);
const search = ref((route.query.search as string) || '');
const page = ref(Number(route.query.page) || 1);

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    data.value = await hostsApi.list({
      page: page.value,
      search: search.value || undefined,
    });
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load hosts';
  } finally {
    loading.value = false;
  }
}

async function handleCreate(formData: unknown) {
  try {
    await hostsApi.create(formData as Parameters<typeof hostsApi.create>[0]);
    showCreateModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create host';
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
      <h1 class="text-2xl font-bold">Hosts</h1>
      <button class="btn btn-primary" @click="showCreateModal = true">
        Add Host
      </button>
    </div>

    <div class="mb-4">
      <form @submit.prevent="handleSearch" class="join">
        <input
          v-model="search"
          type="text"
          placeholder="Search hosts..."
          class="input input-bordered join-item w-64"
        />
        <button type="submit" class="btn join-item">Search</button>
      </form>
    </div>

    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>
    <EmptyState
      v-else-if="!data?.data.length"
      message="No hosts found"
      actionLabel="Add Host"
      @action="showCreateModal = true"
    />

    <div v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Hostname</th>
              <th>IP Address</th>
              <th>OS</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="host in data.data"
              :key="host.id"
              class="hover cursor-pointer"
              @click="router.push(`/hosts/${host.id}`)"
            >
              <td class="font-medium">{{ host.name }}</td>
              <td>{{ host.host_type }}</td>
              <td>{{ host.hostname || '-' }}</td>
              <td>{{ host.ip_address || '-' }}</td>
              <td>{{ host.os || '-' }}</td>
              <td><StatusBadge :status="host.status" /></td>
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
      title="Create Host"
      :open="showCreateModal"
      @close="showCreateModal = false"
    >
      <HostForm @submit="handleCreate" @cancel="showCreateModal = false" />
    </Modal>
  </div>
</template>
