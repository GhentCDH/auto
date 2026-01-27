<script setup lang="ts">
import { sharesApi } from '@/api';
import type { NetworkShare } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import ShareForm from '@/components/forms/ShareForm.vue';
</script>

<template>
  <EntityList
    title="Network Shares"
    add-label="Add Share"
    search-placeholder="Search shares..."
    empty-message="No shares found"
    modal-title="Create Network Share"
    base-path="/shares"
    :fetch-fn="sharesApi.list"
    :create-fn="sharesApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>Type</th>
      <th>Path</th>
      <th>Server</th>
      <th>Status</th>
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
