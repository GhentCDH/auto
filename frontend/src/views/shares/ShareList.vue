<script setup lang="ts">
import { ref } from 'vue';
import { sharesApi } from '@/api';
import type { NetworkShare } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import ShareForm from '@/components/forms/ShareForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import { statusFilterOptions, shareTypeFilterOptions } from '@/values';

const entityListRef = ref<InstanceType<typeof EntityList> | null>(null);
const filters = ref<Record<string, string | null>>({
  status: null,
  share_type: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Storage"
    add-label="Add Storag"
    search-placeholder="Search storage..."
    empty-message="No storage found"
    modal-title="Create Storage"
    base-path="/shares"
    :fetch-fn="sharesApi.list"
    :create-fn="sharesApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #columns>
      <th>Name</th>
      <th>
        Type
        <ColumnFilter
          :options="shareTypeFilterOptions"
          :model-value="filters.share_type"
          @update:model-value="onFilterChange('share_type', $event)"
        />
      </th>
      <th>Path</th>
      <th>Server</th>
      <th>
        Status
        <ColumnFilter
          :options="statusFilterOptions"
          :model-value="filters.status"
          @update:model-value="onFilterChange('status', $event)"
        />
      </th>
    </template>

    <template #row="{ item }: { item: NetworkShare }">
      <td class="font-medium">{{ item.name }}</td>
      <td>{{ item.share_type.toUpperCase() }}</td>
      <td class="font-mono text-sm">{{ item.path }}</td>
      <td>{{ item.server || '-' }}</td>
      <td><StatusBadge :status="item.status" /></td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <ShareForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
