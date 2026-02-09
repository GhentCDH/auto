<script setup lang="ts">
import { useRouter } from 'vue-router';
import { sharesApi } from '@/api';
import type { NetworkShareWithRelations } from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import ShareForm from '@/components/forms/ShareForm.vue';

const router = useRouter();
</script>

<template>
  <EntityDetail
    entity-name="Storage"
    list-path="/shares"
    list-label="Storage"
    :fetch-fn="
      sharesApi.get as (id: string) => Promise<{ id: string; name: string }>
    "
    :update-fn="
      sharesApi.update as (id: string, data: unknown) => Promise<unknown>
    "
    :delete-fn="sharesApi.delete"
  >
    <template #status="{ entity }">
      <StatusBadge :status="(entity as NetworkShareWithRelations).status" />
    </template>

    <template #details="{ entity }">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-sm text-base-content/70">Type</div>
          <div>
            {{ (entity as NetworkShareWithRelations).share_type.toUpperCase() }}
          </div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Server</div>
          <div>{{ (entity as NetworkShareWithRelations).server || '-' }}</div>
        </div>
        <div class="col-span-2">
          <div class="text-sm text-base-content/70">Path</div>
          <div class="font-mono">
            {{ (entity as NetworkShareWithRelations).path }}
          </div>
        </div>
        <div class="col-span-2">
          <div class="text-sm text-base-content/70">Purpose</div>
          <div>{{ (entity as NetworkShareWithRelations).purpose || '-' }}</div>
        </div>
      </div>
      <div v-if="(entity as NetworkShareWithRelations).notes" class="mt-4">
        <div class="text-sm text-base-content/70">Notes</div>
        <div>{{ (entity as NetworkShareWithRelations).notes }}</div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title">
        Applications ({{
          (entity as NetworkShareWithRelations).applications.length
        }})
      </h2>
      <div
        v-if="(entity as NetworkShareWithRelations).applications.length === 0"
        class="text-base-content/70"
      >
        No applications linked
      </div>
      <div v-else class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <th>Name</th>
              <th>Usage</th>
              <th>Mount Point</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="a in (entity as NetworkShareWithRelations).applications"
              :key="a.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${a.id}`)"
            >
              <td>{{ a.name }}</td>
              <td>{{ a.usage || '-' }}</td>
              <td class="font-mono text-xs">{{ a.mount_point || '-' }}</td>
              <td><StatusBadge :status="a.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <ShareForm
        :share="entity as NetworkShareWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
