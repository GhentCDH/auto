<script setup lang="ts">
import { applicationsApi } from '@/api';
import type { Application } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';
</script>

<template>
  <EntityList
    title="Applications"
    add-label="Add Application"
    search-placeholder="Search applications..."
    empty-message="No applications found"
    modal-title="Create Application"
    base-path="/applications"
    :fetch-fn="applicationsApi.list"
    :create-fn="applicationsApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Description</th>
      <th>Status</th>
      <th>Repository</th>
    </template>

    <template #row="{ item }: { item: Application }">
      <td class="font-medium">{{ item.name }}</td>
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
      <ApplicationForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template
