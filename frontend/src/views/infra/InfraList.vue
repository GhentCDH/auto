<script setup lang="ts">
import { ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { infraApi } from '@/api';
import type { Infra } from '@/types';
import { useUptime } from '@/composables/useUptime';
import EntityList from '@/components/common/EntityList.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import HealthStats from '@/components/common/HealthStats.vue';
import { infraTypes, infraTypeFilterOptions } from '@/values';

useUptime();

const entityListRef = ref<{
  updateFilter: (key: string, value: string | null) => void;
} | null>(null);
const filters = ref<Record<string, string | null>>({
  type: null,
  ip: null,
});

// Local IP filter input — debounced before pushing to EntityList so each
// keystroke doesn't trigger a refetch.
const ipInput = ref<string>(filters.value.ip ?? '');
let ipDebounce: ReturnType<typeof setTimeout> | null = null;

watch(
  () => filters.value.ip,
  (v) => {
    if ((v ?? '') !== ipInput.value) ipInput.value = v ?? '';
  }
);

function onIpInput(e: Event) {
  ipInput.value = (e.target as HTMLInputElement).value;
  if (ipDebounce) clearTimeout(ipDebounce);
  ipDebounce = setTimeout(() => {
    onFilterChange('ip', ipInput.value.trim() || null);
  }, 300);
}

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}

const syncLoading = ref(false);
function syncAllIps() {
  syncLoading.value = true;
  const promise = infraApi.syncAll();
  toast.promise(promise, {
    loading: 'Resolving all infra IPs…',
    success: 'All infra IPs synced',
    error: (e: unknown) => (e instanceof Error ? e.message : 'Sync failed'),
  });
  promise.finally(() => (syncLoading.value = false));
}

function matchesIpFilter(ip: string): boolean {
  const f = (filters.value.ip ?? '').trim();
  return !!f && ip.includes(f);
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Infrastructure"
    add-label="Add Infra"
    search-placeholder="Search infrastructure..."
    empty-message="No infrastructure found"
    modal-title="Create Infra"
    base-path="/infra"
    :fetch-fn="infraApi.list"
    :create-fn="infraApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #toolbar>
      <div class="flex gap-2 mb-4">
        <button
          class="btn btn-outline btn-sm"
          :disabled="syncLoading"
          @click="syncAllIps"
        >
          <span v-if="syncLoading" class="loading loading-spinner loading-sm" />
          {{ syncLoading ? 'Syncing…' : 'Sync all IPs' }}
        </button>
      </div>
    </template>

    <template #columns>
      <th>Name</th>
      <th>Description</th>
      <th>
        Type
        <ColumnFilter
          :options="infraTypeFilterOptions"
          :model-value="filters.type"
          @update:model-value="onFilterChange('type', $event)"
        />
      </th>
      <th>
        <div class="flex flex-col gap-1">
          <span>IP</span>
          <input
            type="text"
            placeholder="filter… e.g. 244.44"
            class="input input-bordered input-xs font-normal w-32"
            :value="ipInput"
            @click.stop
            @input="onIpInput"
          />
        </div>
      </th>
      <th>Uptime</th>
    </template>

    <template #row="{ item }: { item: Infra }">
      <td class="font-medium">{{ item.name }}</td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>
      <td>
        {{ infraTypes[item.type as keyof typeof infraTypes] || item.type }}
      </td>
      <td class="font-mono text-xs">
        <div v-if="item.ips && item.ips.length" class="flex flex-wrap gap-1">
          <span
            v-for="ipRow in item.ips"
            :key="ipRow.ip"
            class="badge badge-sm"
            :class="
              matchesIpFilter(ipRow.ip) ? 'badge-primary' : 'badge-ghost'
            "
            :title="`${ipRow.source} · ${ipRow.last_synced_at}`"
          >
            {{ ipRow.ip }}
          </span>
        </div>
        <span v-else class="text-base-content/50">-</span>
      </td>
      <td>
        <HealthStats :kuma-ids="item.healthcheck_kuma_ids ?? []" />
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <InfraForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
