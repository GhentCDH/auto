<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import LoadingSpinner from './LoadingSpinner.vue';
import EnvironmentBadge from './EnvironmentBadge.vue';
import { SearchX } from 'lucide-vue-next';

const props = defineProps<{
  title: string;
  fetchFn: (params: { search?: string }) => Promise<{
    data: Array<{ id: string; name: string; environment?: string }>;
  }>;
  excludeIds?: string[];
  allowCreate?: boolean;
}>();

const emit = defineEmits<{
  select: [entity: { id: string; name: string }];
  create: [searchTerm: string];
  cancel: [];
}>();

const loading = ref(true);
const showSpinner = ref(false);
const error = ref('');
const search = ref('');
const entities = ref<Array<{ id: string; name: string; environment?: string }>>(
  []
);
const searchInput = ref<HTMLInputElement | null>(null);

const filteredEntities = computed(() => {
  const excluded = new Set(props.excludeIds || []);
  return entities.value.filter((e) => !excluded.has(e.id));
});

const singularTitle = computed(() =>
  props.title.at(-1) === 's' ? props.title.slice(0, -1) : props.title
);

const canCreate = computed(() => props.allowCreate !== false);

let spinnerTimeout: ReturnType<typeof setTimeout> | null = null;

async function loadEntities() {
  loading.value = true;
  error.value = '';

  if (spinnerTimeout) clearTimeout(spinnerTimeout);
  showSpinner.value = false;

  spinnerTimeout = setTimeout(() => {
    if (loading.value) showSpinner.value = true;
  }, 150);

  try {
    const result = await props.fetchFn({ search: search.value || undefined });
    entities.value = result.data;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load';
  } finally {
    loading.value = false;
    showSpinner.value = false;
    if (spinnerTimeout) clearTimeout(spinnerTimeout);
  }
}

function handleCreate() {
  emit('create', search.value);
}

let debounceTimeout: ReturnType<typeof setTimeout> | null = null;

watch(search, () => {
  if (debounceTimeout) clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(loadEntities, 150);
});

onMounted(() => {
  loadEntities();
  searchInput.value?.focus();
});
</script>

<template>
  <div class="flex flex-col gap-4">
    <input
      ref="searchInput"
      v-model="search"
      type="text"
      :placeholder="`Search ${title.toLowerCase()}...`"
      class="input input-bordered w-full"
    />

    <!-- Fixed height content area to prevent layout shifts -->
    <div class="h-50 flex flex-col">
      <!-- Loading state -->
      <div v-if="loading" class="flex-1 flex items-center justify-center">
        <LoadingSpinner v-if="showSpinner" />
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="alert alert-error text-sm">
        {{ error }}
      </div>

      <!-- Content state -->
      <template v-else>
        <!-- Results list -->
        <ul
          v-if="filteredEntities.length > 0"
          class="menu rounded-box flex-1 min-h-0 overflow-y-auto w-auto"
        >
          <li v-for="entity in filteredEntities" :key="entity.id">
            <button
              type="button"
              @click="emit('select', entity)"
              class="justify-between"
            >
              <span class="flex gap-2">
                {{ entity.name }}
                <EnvironmentBadge
                  v-if="entity.environment"
                  :environment="entity.environment"
                />
              </span>
              <span class="badge badge-ghost badge-sm">Select</span>
            </button>
          </li>
        </ul>

        <!-- Empty state -->
        <div
          v-else
          class="flex-1 flex flex-col items-center justify-center py-6 text-base-content/50"
        >
          <SearchX class="w-10 h-10 mb-3 opacity-50" />
          <p class="text-sm">
            No {{ title.toLowerCase() }} found{{
              search ? ` matching "${search}"` : ''
            }}
          </p>
        </div>
      </template>
    </div>

    <!-- Footer actions - always visible -->
    <div class="flex flex-col gap-2 pt-2 border-t border-base-300">
      <button
        v-if="canCreate"
        type="button"
        class="btn btn-primary btn-sm w-full"
        @click="handleCreate"
      >
        + Create {{ singularTitle }}{{ search ? `: "${search}"` : '' }}
      </button>
      <button type="button" class="btn btn-ghost btn-sm" @click="emit('cancel')">
        Cancel
      </button>
    </div>
  </div>
</template>
