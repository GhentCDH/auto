<script setup lang="ts">
import { ref } from 'vue';
import { toast } from 'vue-sonner';
import { infraApi } from '@/api';
import type { Infra } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import { infraTypes, infraTypeFilterOptions } from '@/values';

const entityListRef = ref<{
  updateFilter: (key: string, value: string | null) => void;
} | null>(null);
const filters = ref<Record<string, string | null>>({
  type: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}

const syncLoading = ref(false);
function syncAllIps() {
  syncLoading.value = true;
  const promise = infraApi.syncAll();
  toast.promise(promise, {
    loading: 'Resolving all infra IPs…',
    success: 'All infra IPs synced',
    error: (e: unknown) => (e instanceof Error ? e.message : 'Sync failed'),
  });
  promise.finally(() => (syncLoading.value = false));
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Infrastructure"
    add-label="Add Infra"
    search-placeholder="Search infrastructure..."
    empty-message="No infrastructure found"
    modal-title="Create Infra"
    base-path="/infra"
    :fetch-fn="infraApi.list"
    :create-fn="infraApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #toolbar>
      <div class="flex gap-2 mb-4">
        <button
          class="btn btn-outline btn-sm"
          :disabled="syncLoading"
          @click="syncAllIps"
        >
          <span v-if="syncLoading" class="loading loading-spinner loading-sm" />
          {{ syncLoading ? 'Syncing…' : 'Sync all IPs' }}
        </button>
      </div>
    </template>

    <template #columns>
      <th>Name</th>
      <th>Description</th>
      <th>
        Type
        <ColumnFilter
          :options="infraTypeFilterOptions"
          :model-value="filters.type"
          @update:model-value="onFilterChange('type', $event)"
        />
      </th>
    </template>

    <template #row="{ item }: { item: Infra }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>
      <td>
        {{ infraTypes[item.type as keyof typeof infraTypes] || item.type }}
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <InfraForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
