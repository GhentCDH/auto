<script setup lang="ts">
import { useRouter } from 'vue-router';
import { hostsApi } from '@/api';
import type { HostWithRelations } from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HostForm from '@/components/forms/HostForm.vue';

const router = useRouter();
</script>

<template>
  <EntityDetail
    entity-name="Host"
    list-path="/hosts"
    :fetch-fn="
      hostsApi.get as (id: string) => Promise<{ id: string; name: string }>
    "
    :update-fn="
      hostsApi.update as (id: string, data: unknown) => Promise<unknown>
    "
    :delete-fn="hostsApi.delete"
  >
    <template #status="{ entity }">
      <StatusBadge :status="(entity as HostWithRelations).status" />
    </template>

    <template #details="{ entity }">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-sm text-base-content/70">Type</div>
          <div>{{ (entity as HostWithRelations).host_type }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Hostname</div>
          <div>{{ (entity as HostWithRelations).hostname || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">IP Address</div>
          <div>{{ (entity as HostWithRelations).ip_address || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Location</div>
          <div>{{ (entity as HostWithRelations).location || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">OS</div>
          <div>{{ (entity as HostWithRelations).os || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Specs</div>
          <div>{{ (entity as HostWithRelations).specs || '-' }}</div>
        </div>
      </div>
      <div v-if="(entity as HostWithRelations).notes" class="mt-4">
        <div class="text-sm text-base-content/70">Notes</div>
        <div>{{ (entity as HostWithRelations).notes }}</div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title">
        Applications ({{ (entity as HostWithRelations).applications.length }})
      </h2>
      <div
        v-if="(entity as HostWithRelations).applications.length === 0"
        class="text-base-content/70"
      >
        No applications linked
      </div>
      <div v-else class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <th>Name</th>
              <th>Role</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="a in (entity as HostWithRelations).applications"
              :key="a.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${a.id}`)"
            >
              <td>{{ a.name }}</td>
              <td>
                <span class="badge badge-outline">{{ a.role }}</span>
              </td>
              <td><StatusBadge :status="a.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <HostForm
        :host="entity as HostWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
