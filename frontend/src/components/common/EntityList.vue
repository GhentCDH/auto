<script setup lang="ts" generic="T extends { id: string }, C = unknown">
import { ref, watch, computed } from 'vue';
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
  fetchFn: (params: Record<string, unknown>) => Promise<PaginatedResponse<T>>;
  createFn?: (data: C) => Promise<T>;
  filters?: Record<string, string | null>;
}>();

const emit = defineEmits<{
  created: [];
  'update:filters': [filters: Record<string, string | null>];
}>();

const route = useRoute();
const router = useRouter();

const data = ref<PaginatedResponse<T> | null>(null);
const loading = ref(true);
const error = ref('');
const showCreateModal = ref(false);
const search = ref((route.query.search as string) || '');
const page = ref(Number(route.query.page) || 1);

// Initialize filters from URL query params
const localFilters = ref<Record<string, string | null>>({});

// Compute active filters (excluding null values)
const activeFilters = computed(() => {
  const filters: Record<string, string> = {};
  for (const [key, value] of Object.entries(localFilters.value)) {
    if (value !== null && value !== '') {
      filters[key] = value;
    }
  }
  return filters;
});

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    data.value = await props.fetchFn({
      page: page.value,
      search: search.value || undefined,
      ...activeFilters.value,
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
  syncToUrl();
}

function navigateToDetail(id: string) {
  router.push(`${props.basePath}/${id}`);
}

function updateFilter(key: string, value: string | null) {
  localFilters.value = { ...localFilters.value, [key]: value };
  page.value = 1; // Reset to page 1 when filter changes
  syncToUrl();
}

function syncToUrl() {
  const query: Record<string, string | undefined> = {
    ...route.query,
    search: search.value || undefined,
    page: page.value > 1 ? String(page.value) : undefined,
  };
  // Add active filters to query
  for (const [key, value] of Object.entries(localFilters.value)) {
    query[key] = value || undefined;
  }
  router.push({ query });
}

// Initialize filters from URL and props
function initFiltersFromUrl() {
  const filterKeys = props.filters ? Object.keys(props.filters) : [];
  const newFilters: Record<string, string | null> = {};
  for (const key of filterKeys) {
    newFilters[key] = (route.query[key] as string) || null;
  }
  localFilters.value = newFilters;
}

function resetFilters() {
  for (const key in localFilters.value) {
    updateFilter(key, null);
  }
  syncToUrl();
  // Emit the reset filters back to parent
  emit('update:filters', localFilters.value);
}

function filtersActive(): boolean {
  return Object.values(localFilters.value).some((value) => value !== null);
}

watch(
  () => route.query,
  () => {
    page.value = Number(route.query.page) || 1;
    search.value = (route.query.search as string) || '';
    // Update filters from URL
    if (props.filters) {
      for (const key of Object.keys(props.filters)) {
        localFilters.value[key] = (route.query[key] as string) || null;
      }
    }
    loadData();
  },
  { immediate: true }
);

// Initialize filters when component mounts
watch(
  () => props.filters,
  () => {
    initFiltersFromUrl();
  },
  { immediate: true }
);

defineExpose({ loadData, updateFilter, localFilters });
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

    <!-- Optional toolbar (extra actions above search) -->
    <slot name="toolbar" />

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

      <button
        v-if="!data?.data.length && filtersActive()"
        class="ml-4 btn btn-primary"
        @click="resetFilters"
      >
        Reset filters
      </button>
    </div>

    <!-- Loading -->
    <LoadingSpinner v-if="loading" />

    <!-- Error -->
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <!-- Empty -->
    <div v-else-if="!data?.data.length">
      <EmptyState
        :message="emptyMessage"
        :action-label="addLabel"
        @action="showCreateModal = true"
      />
    </div>

    <!-- Table -->
    <div v-else>
      <div class="overflow-x-auto">
        <table class="table">
          <thead>
            <tr>
              <slot name="columns" />
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(item, index) in data.data"
              :key="item.id"
              class="hover:bg-base-200 transition-colors duration-75 cursor-pointer"
              :class="index % 2 === 0 ? '' : 'bg-black/10'"
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
