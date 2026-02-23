<script setup lang="ts">
import { ref } from 'vue';
import { servicesApi } from '@/api';
import type { Service } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import { statusFilterOptions, environmentFilterOptions } from '@/values';

const entityListRef = ref<{
  updateFilter: (key: string, value: string | null) => void;
} | null>(null);
const filters = ref<Record<string, string | null>>({
  status: null,
  environment: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Services"
    add-label="Add Service"
    search-placeholder="Search services..."
    empty-message="No services found"
    modal-title="Create Service"
    base-path="/services"
    :fetch-fn="servicesApi.list"
    :create-fn="servicesApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #columns>
      <th class="name-env">
        Name
        <div>
          <ColumnFilter
            :options="environmentFilterOptions"
            :model-value="filters.environment"
            @update:model-value="onFilterChange('environment', $event)"
          />
          <span class="ml-2 badge badge-sm badge-neutral">env</span>
        </div>
      </th>
      <th class="w-full">Description</th>
      <th>
        Status
        <ColumnFilter
          :options="statusFilterOptions"
          :model-value="filters.status"
          @update:model-value="onFilterChange('status', $event)"
        />
      </th>
      <th>Repository</th>
    </template>

    <template #row="{ item }: { item: Service }">
      <td class="font-medium name-env">
        {{ item.name }}
        <EnvironmentBadge :environment="item.environment" />
      </td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>

      <td><StatusBadge :status="item.status" /></td>
      <td>
        <a
          v-if="item.repository_url"
          :href="item.repository_url"
          target="_blank"
          class="link link-primary"
          @click.stop
          >Link</a
        >
        <span v-else>-</span>
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <ServiceForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
