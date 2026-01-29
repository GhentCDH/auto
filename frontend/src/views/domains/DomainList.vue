<script setup lang="ts">
import { ref } from 'vue';
import { domainsApi } from '@/api';
import type { Domain } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import { domainStatusFilterOptions } from '@/values';

const entityListRef = ref<InstanceType<typeof EntityList> | null>(null);
const filters = ref<Record<string, string | null>>({
  status: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Domains"
    add-label="Add Domain"
    search-placeholder="Search domains..."
    empty-message="No domains found"
    modal-title="Create Domain"
    base-path="/domains"
    :fetch-fn="domainsApi.list"
    :create-fn="domainsApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #columns>
      <th>Domain</th>
      <th>Registrar</th>
      <th>Expires</th>
      <th>
        Status
        <ColumnFilter
          :options="domainStatusFilterOptions"
          :model-value="filters.status"
          @update:model-value="onFilterChange('status', $event)"
        />
      </th>
    </template>

    <template #row="{ item }: { item: Domain }">
      <td class="font-medium">{{ item.name }}</td>
      <td>{{ item.registrar || '-' }}</td>
      <td>{{ item.expires_at || '-' }}</td>
      <td><StatusBadge :status="item.status" /></td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <DomainForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
