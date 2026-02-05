<script setup lang="ts">
import { ref } from 'vue';
import { healthchecksApi } from '@/api';
import type { HealthcheckWithRelations } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';

const exportLoading = ref(false);
const exportError = ref('');

async function exportKuma() {
  exportLoading.value = true;
  exportError.value = '';
  try {
    const monitors = await healthchecksApi.exportKuma();
    const blob = new Blob([JSON.stringify(monitors, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'kuma-monitors.json';
    a.click();
    URL.revokeObjectURL(url);
  } catch (e) {
    exportError.value = e instanceof Error ? e.message : 'Export failed';
  } finally {
    exportLoading.value = false;
  }
}

function buildUrl(hc: HealthcheckWithRelations): string {
  return `${hc.protocol}://${hc.domain_fqdn}${hc.path}`;
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Healthchecks"
    add-label="Add Healthcheck"
    search-placeholder="Search healthchecks..."
    empty-message="No healthchecks found"
    modal-title="Create Healthcheck"
    base-path="/healthchecks"
    :fetch-fn="healthchecksApi.list"
    :create-fn="healthchecksApi.create"
  >
    <template #columns>
      <th>Name</th>
      <th>URL</th>
      <th>Target</th>
      <th>Method</th>
      <th>Expected</th>
      <th>Status</th>
    </template>

    <template #row="{ item }: { item: HealthcheckWithRelations }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="text-sm font-mono truncate max-w-xs">{{ buildUrl(item) }}</td>
      <td>
        <router-link
          v-if="item.application_id"
          :to="`/applications/${item.application_id}`"
          class="link link-hover"
          @click.stop
        >
          {{ item.application_name }}
        </router-link>
        <router-link
          v-else-if="item.service_id"
          :to="`/services/${item.service_id}`"
          class="link link-hover"
          @click.stop
        >
          {{ item.service_name }}
        </router-link>
      </td>
      <td>
        <span class="badge badge-outline badge-sm">{{ item.method }}</span>
      </td>
      <td>{{ item.expected_status }}</td>
      <td>
        <StatusBadge :status="item.is_enabled ? 'active' : 'inactive'" />
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <HealthcheckForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>

  <!-- Export Kuma button -->
  <div class="fixed bottom-6 right-6">
    <button
      class="btn btn-secondary shadow-lg"
      :disabled="exportLoading"
      @click="exportKuma"
    >
      <span v-if="exportLoading" class="loading loading-spinner loading-sm" />
      Export Kuma
    </button>
  </div>

  <!-- Export error toast -->
  <div v-if="exportError" class="toast toast-end">
    <div class="alert alert-error">
      <span>{{ exportError }}</span>
      <button class="btn btn-ghost btn-xs" @click="exportError = ''">
        &times;
      </button>
    </div>
  </div>
</template>
