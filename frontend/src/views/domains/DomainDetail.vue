<script setup lang="ts">
import { useRouter } from 'vue-router';
import { domainsApi } from '@/api';
import type { DomainWithRelations } from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import DomainForm from '@/components/forms/DomainForm.vue';

const router = useRouter();
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
    <template #status="{ entity }">
      <StatusBadge :status="(entity as DomainWithRelations).status" />
    </template>

    <template #details="{ entity }">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-sm text-base-content/70">Registrar</div>
          <div>{{ (entity as DomainWithRelations).registrar || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">DNS Provider</div>
          <div>{{ (entity as DomainWithRelations).dns_provider || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Expires</div>
          <div>{{ (entity as DomainWithRelations).expires_at || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">SSL Expires</div>
          <div>{{ (entity as DomainWithRelations).ssl_expires_at || '-' }}</div>
        </div>
        <div class="col-span-2">
          <div class="text-sm text-base-content/70">SSL Issuer</div>
          <div>{{ (entity as DomainWithRelations).ssl_issuer || '-' }}</div>
        </div>
      </div>
      <div v-if="(entity as DomainWithRelations).notes" class="mt-4">
        <div class="text-sm text-base-content/70">Notes</div>
        <div>{{ (entity as DomainWithRelations).notes }}</div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title">
        Applications ({{ (entity as DomainWithRelations).applications.length }})
      </h2>
      <div
        v-if="(entity as DomainWithRelations).applications.length === 0"
        class="text-base-content/70"
      >
        No applications linked
      </div>
      <div v-else class="overflow-x-auto">
        <table class="table table-sm">
          <thead>
            <tr>
              <th>Name</th>
              <th>Record Type</th>
              <th>Primary</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="a in (entity as DomainWithRelations).applications"
              :key="a.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${a.id}`)"
            >
              <td>{{ a.name }}</td>
              <td>{{ a.record_type }}</td>
              <td>{{ a.is_primary ? 'Yes' : 'No' }}</td>
              <td><StatusBadge :status="a.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <DomainForm
        :domain="entity as DomainWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
