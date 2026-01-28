<script setup lang="ts">
import { servicesApi } from '@/api';
import type { Service } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
</script>

<template>
  <EntityList
    title="Services"
    add-label="Add Service"
    search-placeholder="Search services..."
    empty-message="No services found"
    modal-title="Create Service"
    base-path="/services"
    :fetch-fn="servicesApi.list"
    :create-fn="servicesApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Description</th>
      <th>Environment</th>
      <th>Status</th>
      <th>Repository</th>
    </template>

    <template #row="{ item }: { item: Service }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>
      <td><EnvironmentBadge :environment="item.environment" /></td>
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
