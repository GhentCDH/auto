<script setup lang="ts">
import { ref } from 'vue';
import { peopleApi } from '@/api';
import type { Person } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import PersonForm from '@/components/forms/PersonForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import { personActiveFilterOptions } from '@/values';

const entityListRef = ref<{ updateFilter: (key: string, value: string | null) => void } | null>(null);
const filters = ref<Record<string, string | null>>({
  is_active: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="People"
    add-label="Add Person"
    search-placeholder="Search people..."
    empty-message="No people found"
    modal-title="Create Person"
    base-path="/people"
    :fetch-fn="peopleApi.list"
    :create-fn="peopleApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #columns>
      <th>Name</th>
      <th>Email</th>
      <th>Role</th>
      <th>Department</th>
      <th>
        Status
        <ColumnFilter
          :options="personActiveFilterOptions"
          :model-value="filters.is_active"
          @update:model-value="onFilterChange('is_active', $event)"
        />
      </th>
    </template>

    <template #row="{ item }: { item: Person }">
      <td class="font-medium">{{ item.name }}</td>
      <td>{{ item.email || '-' }}</td>
      <td>{{ item.role || '-' }}</td>
      <td>{{ item.department || '-' }}</td>
      <td>
        <span
          class="badge"
          :class="item.is_active ? 'badge-success' : 'badge-warning'"
          >{{ item.is_active ? 'Active' : 'Inactive' }}</span
        >
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <PersonForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
