<script setup lang="ts">
import { useRouter } from 'vue-router';
import { domainsApi } from '@/api';
import type { DomainNamedWithRelations, DomainWithRelations } from '@/types';
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
