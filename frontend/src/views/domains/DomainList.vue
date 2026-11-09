<script setup lang="ts">
import { ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { domainsApi } from '@/api';
import type { Domain, DomainIp } from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import DomainForm from '@/components/forms/DomainForm.vue';

const entityListRef = ref<{
  updateFilter: (key: string, value: string | null) => void;
} | null>(null);
const filters = ref<Record<string, string | null>>({ ip: null });

// Debounced so each keystroke doesn't trigger a refetch.
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
    const value = ipInput.value.trim() || null;
    filters.value.ip = value;
    entityListRef.value?.updateFilter('ip', value);
  }, 300);
}

function matchesIpFilter(ip: string): boolean {
  const f = (filters.value.ip ?? '').trim();
  return !!f && ip.includes(f);
}

const syncLoading = ref(false);
function syncAllIps() {
  syncLoading.value = true;
  const promise = domainsApi.syncAll();
  toast.promise(promise, {
    loading: 'Resolving all domain IPs…',
    success: 'All domain IPs synced',
    error: (e: unknown) => (e instanceof Error ? e.message : 'Sync failed'),
  });
  promise.finally(() => (syncLoading.value = false));
}
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
      <th>Domain</th>
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
      <th class="hidden md:table-cell">Registrar</th>
      <th class="hidden md:table-cell">Expires</th>
      <th>Target</th>
    </template>

    <template #row="{ item }: { item: Domain & { ips?: DomainIp[] } }">
      <td class="font-medium">{{ item.fqdn }}</td>
      <td class="font-mono text-xs">
        <div v-if="item.ips && item.ips.length" class="flex flex-wrap gap-1">
          <span
            v-for="ipRow in item.ips"
            :key="ipRow.ip"
            class="badge badge-sm"
            :class="matchesIpFilter(ipRow.ip) ? 'badge-primary' : 'badge-ghost'"
            :title="
              ipRow.via === 'CNAME'
                ? `via CNAME ${ipRow.cname} · ${ipRow.last_synced_at}`
                : `A · ${ipRow.last_synced_at}`
            "
          >
            {{ ipRow.ip }}
            <span v-if="ipRow.via === 'CNAME'" class="opacity-60">(CNAME)</span>
          </span>
        </div>
        <span v-else class="text-base-content/50">-</span>
      </td>
      <td class="hidden md:table-cell">{{ item.registrar || '-' }}</td>
      <td class="hidden md:table-cell">{{ item.expires_at || '-' }}</td>
      <td v-if="item.target_application_id">
        <router-link :to="`/applications/${item.target_application_id}`">{{
          item.target_application_name
        }}</router-link>
      </td>
      <td v-else-if="item.target_service_id">
        <router-link :to="`/services/${item.target_service_id}`">{{
          item.target_service_name
        }}</router-link>
      </td>
      <td v-else-if="item.target_infra_id">
        <router-link :to="`/infra/${item.target_infra_id}`">{{
          item.target_infra_name
        }}</router-link>
      </td>
      <td v-else>-</td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <DomainForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>
</template>
