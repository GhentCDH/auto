<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import LoadingSpinner from './LoadingSpinner.vue';

const props = defineProps<{
  title: string;
  fetchFn: (params: {
    search?: string;
  }) => Promise<{ data: Array<{ id: string; name: string }> }>;
  excludeIds?: string[];
  allowCreate?: boolean;
}>();

const emit = defineEmits<{
  select: [entity: { id: string; name: string }];
  create: [searchTerm: string];
  cancel: [];
}>();

const loading = ref(true);
const error = ref('');
const search = ref('');
const entities = ref<Array<{ id: string; name: string }>>([]);

const filteredEntities = computed(() => {
  const excluded = new Set(props.excludeIds || []);
  return entities.value.filter((e) => !excluded.has(e.id));
});

const showCreateOption = computed(() => {
  return (
    props.allowCreate !== false &&
    !loading.value &&
    filteredEntities.value.length === 0
  );
});

async function loadEntities() {
  loading.value = true;
  error.value = '';
  try {
    const result = await props.fetchFn({ search: search.value || undefined });
    entities.value = result.data;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load';
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  loadEntities();
}

function handleCreate() {
  emit('create', search.value);
}

onMounted(loadEntities);
</script>

<template>
  <div class="space-y-4">
    <form @submit.prevent="handleSearch" class="join w-full">
      <input
        v-model="search"
        type="text"
        :placeholder="`Search ${title.toLowerCase()}...`"
        class="input input-bordered join-item flex-1"
      />
      <button type="submit" class="btn join-item">Search</button>
    </form>

    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error text-sm">{{ error }}</div>

    <div v-else-if="filteredEntities.length === 0" class="text-center py-6">
      <p class="text-base-content/70 mb-4">
        No {{ title.toLowerCase() }} found{{
          search ? ` matching "${search}"` : ''
        }}
      </p>
      <button
        v-if="showCreateOption"
        type="button"
        class="btn btn-primary btn-sm"
        @click="handleCreate"
      >
        + Create New {{ title.slice(0, -1) }}{{ search ? `: "${search}"` : '' }}
      </button>
    </div>

    <template v-else>
      <ul class="menu bg-base-100 rounded-box max-h-64 overflow-y-auto">
        <li v-for="entity in filteredEntities" :key="entity.id">
          <a @click="emit('select', entity)" class="justify-between">
            {{ entity.name }}
            <span class="badge badge-ghost badge-sm">Select</span>
          </a>
        </li>
      </ul>
      <button
        v-if="allowCreate !== false"
        type="button"
        class="btn btn-ghost btn-sm w-full"
        @click="handleCreate"
      >
        + Create New {{ title.slice(0, -1) }}
      </button>
    </template>

    <div class="flex justify-end">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
    </div>
  </div>
</template>
