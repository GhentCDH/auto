<script setup lang="ts">
import { domainsApi } from '@/api';
import type { Domain } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
</script>

<template>
  <EntityList
    title="Domains"
    add-label="Add Domain"
    search-placeholder="Search domains..."
    empty-message="No domains found"
    modal-title="Create Domain"
    base-path="/domains"
    :fetch-fn="domainsApi.list"
    :create-fn="domainsApi.create"
  >
    <template #columns>
      <th>Domain</th>
      <th>Registrar</th>
      <th>Expires</th>
      <th>SSL Expires</th>
      <th>Status</th>
    </template>

    <template #row="{ item }: { item: Domain }">
      <td class="font-medium">{{ item.name }}</td>
      <td>{{ item.registrar || '-' }}</td>
      <td>{{ item.expires_at || '-' }}</td>
      <td>{{ item.ssl_expires_at || '-' }}</td>
      <td><StatusBadge :status="item.status" /></td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <DomainForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
