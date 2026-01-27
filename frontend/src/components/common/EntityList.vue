<script setup lang="ts" generic="T extends { id: string }, C = unknown">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import type { PaginatedResponse } from '@/types';
import LoadingSpinner from './LoadingSpinner.vue';
import EmptyState from './EmptyState.vue';
import Pagination from './Pagination.vue';
import Modal from './Modal.vue';

const props = defineProps<{
  title: string;
  addLabel: string;
  searchPlaceholder: string;
  emptyMessage: string;
  modalTitle: string;
  basePath: string;
  fetchFn: (params: {
    page?: number;
    search?: string;
  }) => Promise<PaginatedResponse<T>>;
  createFn?: (data: C) => Promise<T>;
}>();

const emit = defineEmits<{
  created: [];
}>();

const route = useRoute();
const router = useRouter();

const data = ref<PaginatedResponse<T> | null>(null);
const loading = ref(true);
const error = ref('');
const showCreateModal = ref(false);
const search = ref((route.query.search as string) || '');
const page = ref(Number(route.query.page) || 1);

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    data.value = await props.fetchFn({
      page: page.value,
      search: search.value || undefined,
    });
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load data';
  } finally {
    loading.value = false;
  }
}

async function handleCreate(formData: C) {
  if (!props.createFn) return;
  try {
    await props.createFn(formData);
    showCreateModal.value = false;
    loadData();
    emit('created');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create';
  }
}

// Wrapper that accepts broader form data types (forms may emit Create | Update unions)
function onFormSubmit(formData: unknown) {
  return handleCreate(formData as C);
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

function navigateToDetail(id: string) {
  router.push(`${props.basePath}/${id}`);
}

watch(
  () => route.query,
  () => {
    page.value = Number(route.query.page) || 1;
    search.value = (route.query.search as string) || '';
    loadData();
  },
  { immediate: true }
);

defineExpose({ loadData });
</script>

<template>
  <div class="p-6">
    <!-- Header -->
    <div
      class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
    >
      <h1 class="text-2xl font-bold">{{ title }}</h1>
      <button class="btn btn-primary" @click="showCreateModal = true">
        {{ addLabel }}
      </button>
    </div>

    <!-- Search -->
    <div class="mb-4">
      <form @submit.prevent="handleSearch" class="join">
        <input
          v-model="search"
          type="text"
          :placeholder="searchPlaceholder"
          class="input input-bordered join-item w-64"
        />
        <button type="submit" class="btn join-item">Search</button>
      </form>
    </div>

    <!-- Loading -->
    <LoadingSpinner v-if="loading" />

    <!-- Error -->
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <!-- Empty -->
    <EmptyState
      v-else-if="!data?.data.length"
      :message="emptyMessage"
      :action-label="addLabel"
      @action="showCreateModal = true"
    />

    <!-- Table -->
    <div v-else>
      <div class="overflow-x-auto">
        <table class="table table-zebra">
          <thead>
            <tr>
              <slot name="columns" />
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in data.data"
              :key="item.id"
              class="hover cursor-pointer"
              @click="navigateToDetail(item.id)"
            >
              <slot name="row" :item="item" />
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

    <!-- Create Modal -->
    <Modal
      :title="modalTitle"
      :open="showCreateModal"
      @close="showCreateModal = false"
    >
      <slot
        name="form"
        :on-submit="onFormSubmit"
        :on-cancel="() => (showCreateModal = false)"
      />
    </Modal>
  </div>
</template>
