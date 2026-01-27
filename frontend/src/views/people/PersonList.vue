<script setup lang="ts">
import { peopleApi } from '@/api';
import type { Person } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import PersonForm from '@/components/forms/PersonForm.vue';
</script>

<template>
  <EntityList
    title="People"
    add-label="Add Person"
    search-placeholder="Search people..."
    empty-message="No people found"
    modal-title="Create Person"
    base-path="/people"
    :fetch-fn="peopleApi.list"
    :create-fn="peopleApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Email</th>
      <th>Role</th>
      <th>Department</th>
      <th>Status</th>
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
