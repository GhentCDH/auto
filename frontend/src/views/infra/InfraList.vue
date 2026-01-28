<script setup lang="ts">
import { infraApi } from '@/api';
import type { Infra } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import { infraTypes } from '@/values';
</script>

<template>
  <EntityList
    title="Infrastructure"
    add-label="Add Infra"
    search-placeholder="Search infrastructure..."
    empty-message="No infrastructure found"
    modal-title="Create Infra"
    base-path="/infra"
    :fetch-fn="infraApi.list"
    :create-fn="infraApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Description</th>
      <th>Type</th>
    </template>

    <template #row="{ item }: { item: Infra }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>
      <td>{{ infraTypes[item.type as keyof typeof infraTypes] || item.type }}</td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <InfraForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
