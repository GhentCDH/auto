<script setup lang="ts">
import { useRouter } from 'vue-router';
import { peopleApi } from '@/api';
import type { PersonWithRelations } from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import PersonForm from '@/components/forms/PersonForm.vue';

const router = useRouter();
</script>

<template>
  <EntityDetail
    entity-name="Person"
    list-label="People"
    list-path="/people"
    :fetch-fn="
      peopleApi.get as (id: string) => Promise<{ id: string; name: string }>
    "
    :update-fn="
      peopleApi.update as (id: string, data: unknown) => Promise<unknown>
    "
    :delete-fn="peopleApi.delete"
  >
    <template #status="{ entity }">
      <span
        class="badge"
        :class="
          (entity as PersonWithRelations).is_active
            ? 'badge-success'
            : 'badge-warning'
        "
      >
        {{ (entity as PersonWithRelations).is_active ? 'Active' : 'Inactive' }}
      </span>
    </template>

    <template #details-title>Contact Information</template>

    <template #details="{ entity }">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-sm text-base-content/70">Email</div>
          <div>{{ (entity as PersonWithRelations).email || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Phone</div>
          <div>{{ (entity as PersonWithRelations).phone || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Role</div>
          <div>{{ (entity as PersonWithRelations).role || '-' }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Department</div>
          <div>{{ (entity as PersonWithRelations).department || '-' }}</div>
        </div>
      </div>
      <div v-if="(entity as PersonWithRelations).notes" class="mt-4">
        <div class="text-sm text-base-content/70">Notes</div>
        <div>{{ (entity as PersonWithRelations).notes }}</div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title">
        Applications ({{ (entity as PersonWithRelations).applications.length }})
      </h2>
      <div
        v-if="(entity as PersonWithRelations).applications.length === 0"
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
              v-for="a in (entity as PersonWithRelations).applications"
              :key="a.id"
              class="hover cursor-pointer"
              @click="router.push(`/applications/${a.id}`)"
            >
              <td>{{ a.name }}</td>
              <td>
                <span class="badge badge-outline">{{
                  a.contribution_type
                }}</span>
              </td>
              <td><StatusBadge :status="a.status" /></td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <PersonForm
        :person="entity as PersonWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
