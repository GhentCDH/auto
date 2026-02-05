<script setup lang="ts">
import { ref, computed } from 'vue';
import { healthchecksApi } from '@/api';
import type { Healthcheck, HealthcheckWithRelations } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';
import ImportWizard from '@/components/import/ImportWizard.vue';

const exportLoading = ref(false);
const exportError = ref('');

// import modal state
const showImport = ref(false);
const importPanelOpen = ref(false);
const importPanelWidthRem = ref(0);

// Base modal width in rem
const BASE_MODAL_WIDTH_REM = 48;

// Compute modal width style
const modalWidthStyle = computed(() => {
  const width = BASE_MODAL_WIDTH_REM + (importPanelOpen.value ? importPanelWidthRem.value : 0);
  return { width: `${width}rem` };
});

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

// optional: refresh list after import
const entityListRef = ref<InstanceType<typeof EntityList>>();

function handleImported(count: number) {
  showImport.value = false;
  importPanelOpen.value = false;
  importPanelWidthRem.value = 0;
  if (count > 0) {
    entityListRef.value?.reload?.();
  }
}

function handleImportClose() {
  showImport.value = false;
  importPanelOpen.value = false;
  importPanelWidthRem.value = 0;
}

function handlePanelChange(isOpen: boolean, widthRem: number) {
  importPanelOpen.value = isOpen;
  importPanelWidthRem.value = widthRem;
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

    <template #row="{ item }: { item: Healthcheck }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="text-sm font-mono truncate max-w-xs">
        {{ buildUrl(item as HealthcheckWithRelations) }}
      </td>
      <td>
        <router-link
          v-if="item.application_id"
          :to="`/applications/${item.application_id}`"
          class="link link-hover"
          @click.stop
        >
          {{ (item as HealthcheckWithRelations).application_name }}
        </router-link>
        <router-link
          v-else-if="item.service_id"
          :to="`/services/${item.service_id}`"
          class="link link-hover"
          @click.stop
        >
          {{ (item as HealthcheckWithRelations).service_name }}
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

  <dialog class="modal" :class="{ 'modal-open': showImport }">
    <div
      class="modal-box max-w-[90vw] transition-[width] duration-200"
      :style="modalWidthStyle"
    >
      <ImportWizard
        @close="handleImportClose"
        @imported="handleImported"
        @panel-change="handlePanelChange"
      />
    </div>

    <form method="dialog" class="modal-backdrop">
      <button @click="handleImportClose">close</button>
    </form>
  </dialog>

  <div class="fixed bottom-6 right-6 flex flex-col gap-2">
    <button class="btn btn-primary shadow-lg" @click="showImport = true">
      Import Kuma
    </button>

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
