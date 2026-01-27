<script setup lang="ts">
import { hostsApi } from '@/api';
import type { Host } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HostForm from '@/components/forms/HostForm.vue';
</script>

<template>
  <EntityList
    title="Hosts"
    add-label="Add Host"
    search-placeholder="Search hosts..."
    empty-message="No hosts found"
    modal-title="Create Host"
    base-path="/hosts"
    :fetch-fn="hostsApi.list"
    :create-fn="hostsApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Type</th>
      <th>Hostname</th>
      <th>IP Address</th>
      <th>OS</th>
      <th>Status</th>
    </template>

    <template #row="{ item }: { item: Host }">
      <td class="font-medium">{{ item.name }}</td>
      <td>{{ item.host_type }}</td>
      <td>{{ item.hostname || '-' }}</td>
      <td>{{ item.ip_address || '-' }}</td>
      <td>{{ item.os || '-' }}</td>
      <td><StatusBadge :status="item.status" /></td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <HostForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
