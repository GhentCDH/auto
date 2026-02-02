<script setup lang="ts">
import { domainsApi } from '@/api';
import type { Domain } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
</script>

<template>
  <EntityList
    ref="entityListRef"
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
      <th>Target</th>
    </template>

    <template #row="{ item }: { item: Domain }">
      <td class="font-medium">{{ item.fqdn }}</td>
      <td>{{ item.registrar || '-' }}</td>
      <td>{{ item.expires_at || '-' }}</td>
      <td v-if="item.target_application_id">
        <router-link :to="`/applications/${item.target_application_id}`">{{
          item.target_application_name
        }}</router-link>
      </td>
      <td v-else>
        <router-link :to="`/services/${item.target_service_id}`">{{
          item.target_service_name
        }}</router-link>
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <DomainForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
