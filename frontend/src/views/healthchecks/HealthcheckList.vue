<script setup lang="ts">
import { ref, computed } from 'vue';
import { healthchecksApi } from '@/api';
import type { Healthcheck, HealthcheckWithRelations } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';
import ImportWizard from '@/components/import/ImportWizard.vue';
import HealthStats from '@/components/common/HealthStats.vue';

const syncLoading = ref(false);
const syncError = ref('');
const syncSuccess = ref(false);

// import modal state
const showImport = ref(false);
const importPanelOpen = ref(false);
const importPanelWidthRem = ref(0);

// Base modal width in rem
const BASE_MODAL_WIDTH_REM = 48;

// Compute modal width style
const modalWidthStyle = computed(() => {
  const width =
    BASE_MODAL_WIDTH_REM +
    (importPanelOpen.value ? importPanelWidthRem.value : 0);
  return { width: `${width}rem` };
});

async function syncKuma() {
  syncLoading.value = true;
  syncError.value = '';
  syncSuccess.value = false;
  try {
    await healthchecksApi.syncKumaAll();
    syncSuccess.value = true;
    setTimeout(() => (syncSuccess.value = false), 3000);
    // Reload list to clear dirty flags
    entityListRef.value?.loadData?.();
  } catch (e) {
    syncError.value = e instanceof Error ? e.message : 'Sync failed';
  } finally {
    syncLoading.value = false;
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
      <th>Uptime</th>
      <th>Status</th>
    </template>

    <template #row="{ item }: { item: Healthcheck }">
      <td class="font-medium">
        <span class="flex items-center gap-1.5">
          {{ item.name }}
          <span
            v-if="item.kuma_dirty"
            class="inline-block w-2 h-2 rounded-full bg-warning shrink-0"
            title="Not synced to Kuma"
          />
        </span>
      </td>
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
        <HealthStats v-if="item.kuma_id" :kuma-id="item.kuma_id" />
        <span v-else class="text-base-content/30">&mdash;</span>
      </td>
      <td>
        <StatusBadge :status="item.is_enabled ? 'active' : 'inactive'" />
      </td>
    </template>

    <template #toolbar>
      <div class="flex gap-2 mb-4">
        <button
          class="btn btn-primary"
          :disabled="syncLoading"
          @click="syncKuma"
        >
          <span v-if="syncLoading" class="loading loading-spinner loading-sm" />
          {{ syncLoading ? 'Syncing...' : 'Sync Kuma' }}
        </button>
        <button class="btn btn-outline" @click="showImport = true">
          Import Kuma
        </button>
      </div>
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

  <!-- Sync feedback toasts -->
  <div v-if="syncSuccess" class="toast toast-end">
    <div class="alert alert-success">
      <span>Kuma sync completed successfully.</span>
    </div>
  </div>
  <div v-if="syncError" class="toast toast-end">
    <div class="alert alert-error">
      <span>{{ syncError }}</span>
      <button class="btn btn-ghost btn-xs" @click="syncError = ''">
        &times;
      </button>
    </div>
  </div>
</template>
