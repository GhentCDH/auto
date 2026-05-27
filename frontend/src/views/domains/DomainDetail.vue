<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { domainsApi } from '@/api';
import type {
  DnsRecord,
  DomainNamedWithRelations,
  DomainWithRelations,
} from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import DomainForm from '@/components/forms/DomainForm.vue';

const router = useRouter();
const route = useRoute();

// Live DNS records — fetched on demand from the server, not stored anywhere.
const dnsRecords = ref<DnsRecord[]>([]);
const dnsLoading = ref(false);
const dnsError = ref('');
const dnsResolvedAt = ref('');

async function loadDns() {
  const id = route.params.id as string;
  if (!id) return;
  dnsLoading.value = true;
  dnsError.value = '';
  try {
    const lookup = await domainsApi.getDns(id);
    dnsRecords.value = lookup.records;
    dnsResolvedAt.value = lookup.resolved_at;
  } catch (e: unknown) {
    dnsError.value = e instanceof Error ? e.message : 'DNS lookup failed';
  } finally {
    dnsLoading.value = false;
  }
}

// Badge color per record type for quick visual scanning.
function recordBadgeClass(type: string): string {
  switch (type) {
    case 'A':
    case 'AAAA':
      return 'badge-primary';
    case 'CNAME':
      return 'badge-secondary';
    case 'MX':
      return 'badge-accent';
    case 'TXT':
      return 'badge-ghost';
    case 'NS':
      return 'badge-info';
    default:
      return 'badge-neutral';
  }
}

onMounted(loadDns);
</script>

<template>
  <EntityDetail
    entity-name="Domain"
    list-path="/domains"
    :fetch-fn="
      domainsApi.get as (id: string) => Promise<{ id: string; name: string }>
    "
    :update-fn="
      domainsApi.update as (id: string, data: unknown) => Promise<unknown>
    "
    :delete-fn="domainsApi.delete"
  >
    <template #details="{ entity }">
      <div class="grid grid-cols-3 gap-3">
        <div>
          <div class="text-sm text-base-content/70">Registrar</div>
          <div>{{ (entity as DomainNamedWithRelations).registrar || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">DNS Provider</div>
          <div>
            {{ (entity as DomainNamedWithRelations).dns_provider || '-' }}
          </div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Expires</div>
          <div>
            {{ (entity as DomainNamedWithRelations).expires_at || '-' }}
          </div>
        </div>
      </div>
      <div class="mt-4 grid-cols-2">
        <div v-if="(entity as DomainNamedWithRelations).notes">
          <div class="text-sm text-base-content/70">Notes</div>
          <div>{{ (entity as DomainNamedWithRelations).notes }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Target</div>
          <router-link
            v-if="(entity as DomainNamedWithRelations).target_application_id"
            :to="`/applications/${(entity as DomainNamedWithRelations).target_application_id}`"
          >
            {{ (entity as DomainNamedWithRelations).target_application_name }}
          </router-link>
          <router-link
            v-if="(entity as DomainNamedWithRelations).target_service_id"
            :to="`/services/${(entity as DomainNamedWithRelations).target_service_id}`"
          >
            {{ (entity as DomainNamedWithRelations).target_service_name }}
          </router-link>
          <router-link
            v-if="(entity as DomainNamedWithRelations).target_infra_id"
            :to="`/infra/${(entity as DomainNamedWithRelations).target_infra_id}`"
          >
            {{ (entity as DomainNamedWithRelations).target_infra_name }}
          </router-link>
        </div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title">
        Applications ({{
          (entity as DomainNamedWithRelations).applications.length
        }})
      </h2>
      <div
        v-if="(entity as DomainNamedWithRelations).applications.length === 0"
        class="text-base-content/70"
      >
        No applications linked
      </div>
      <div v-else class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <th>Name</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="a in (entity as DomainNamedWithRelations).applications"
              :key="a.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${a.id}`)"
            >
              <td>{{ a.name }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="mt-6 flex items-center justify-between">
        <h2 class="card-title">DNS Records ({{ dnsRecords.length }})</h2>
        <button
          class="btn btn-sm btn-ghost"
          :disabled="dnsLoading"
          @click="loadDns"
        >
          <span v-if="dnsLoading" class="loading loading-spinner loading-xs" />
          Refresh
        </button>
      </div>

      <div v-if="dnsError" class="alert alert-error alert-soft text-sm">
        {{ dnsError }}
      </div>
      <div
        v-else-if="dnsLoading && dnsRecords.length === 0"
        class="text-base-content/70"
      >
        Resolving…
      </div>
      <div
        v-else-if="dnsRecords.length === 0"
        class="text-base-content/70"
      >
        No DNS records found
      </div>
      <div v-else class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <th>Type</th>
              <th class="w-full">Value</th>
              <th>Infra</th>
              <th>TTL</th>
              <th>Priority</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(r, i) in dnsRecords" :key="`${r.record_type}-${i}`">
              <td>
                <span
                  class="badge badge-sm"
                  :class="recordBadgeClass(r.record_type)"
                  >{{ r.record_type }}</span
                >
              </td>
              <td class="font-mono break-all">{{ r.value }}</td>
              <td>
                <router-link
                  v-if="r.infra"
                  :to="`/infra/${r.infra.id}`"
                  class="badge badge-sm badge-success badge-outline"
                  :title="`Matches infra ${r.infra.name}`"
                >
                  {{ r.infra.name }}
                </router-link>
                <span v-else class="text-base-content/30">-</span>
              </td>
              <td>{{ r.ttl }}</td>
              <td>{{ r.priority ?? '-' }}</td>
            </tr>
          </tbody>
        </table>
        <div v-if="dnsResolvedAt" class="mt-2 text-xs text-base-content/50">
          As of {{ new Date(dnsResolvedAt).toLocaleString() }}
        </div>
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <DomainForm
        :domain="entity as DomainNamedWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
