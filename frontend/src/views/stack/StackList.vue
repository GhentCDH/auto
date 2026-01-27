<script setup lang="ts">
import { stacksApi } from '@/api';
import type { Stack } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StackForm from '@/components/forms/StackForm.vue';
import StackBadge from '@/components/common/StackBadge.vue';
</script>

<template>
  <EntityList
    title="Technology Stack"
    add-label="Add Stack Item"
    search-placeholder="Search stack..."
    empty-message="No stack items found"
    modal-title="Create Stack Item"
    base-path="/stack"
    :fetch-fn="stacksApi.list"
    :create-fn="stacksApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Notes</th>
    </template>

    <template #row="{ item }: { item: Stack }">
      <td><StackBadge :name="item.name" /></td>
      <td>{{ item.notes || '-' }}</td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <StackForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
