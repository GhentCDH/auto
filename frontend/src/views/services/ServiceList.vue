<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { servicesApi } from '@/api';
import type {
  Service,
  ServiceWithRelations,
  CreateService,
  UpdateService,
} from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import ColumnFilter from '@/components/common/ColumnFilter.vue';
import Modal from '@/components/common/Modal.vue';
import { statusFilterOptions, environmentFilterOptions } from '@/values';
import { Copy } from 'lucide-vue-next';

const router = useRouter();

const entityListRef = ref<{
  updateFilter: (key: string, value: string | null) => void;
  loadData: () => void;
} | null>(null);
const filters = ref<Record<string, string | null>>({
  status: null,
  environment: null,
});

function onFilterChange(key: string, value: string | null) {
  filters.value[key] = value;
  entityListRef.value?.updateFilter(key, value);
}

// Duplicate state
const duplicateSource = ref<ServiceWithRelations | null>(null);
const showDuplicateModal = ref(false);
const duplicating = ref(false);

async function handleDuplicate(id: string) {
  try {
    duplicateSource.value = await servicesApi.get(id);
    showDuplicateModal.value = true;
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to load service');
  }
}

async function handleDuplicateSubmit(formData: CreateService | UpdateService) {
  if (!duplicateSource.value) return;
  duplicating.value = true;

  try {
    const newSvc = await servicesApi.create(formData as CreateService);
    const source = duplicateSource.value;

    // Copy infra relations (skip healthchecks and application links)
    const relationPromises: Promise<unknown>[] = [];

    for (const infra of source.infra) {
      relationPromises.push(
        servicesApi.linkInfra(newSvc.id, infra.id, {
          notes: infra.relation_notes || undefined,
        })
      );
    }

    await Promise.allSettled(relationPromises);

    showDuplicateModal.value = false;
    duplicateSource.value = null;
    toast.success('Service duplicated successfully');
    router.push(`/services/${newSvc.id}`);
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to duplicate service');
  } finally {
    duplicating.value = false;
  }
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Services"
    add-label="Add Service"
    search-placeholder="Search services..."
    empty-message="No services found"
    modal-title="Create Service"
    base-path="/services"
    :fetch-fn="servicesApi.list"
    :create-fn="servicesApi.create"
    :filters="filters"
    @update:filters="filters = $event"
  >
    <template #columns>
      <th class="name-env">
        Name
        <div>
          <ColumnFilter
            :options="environmentFilterOptions"
            :model-value="filters.environment"
            @update:model-value="onFilterChange('environment', $event)"
          />
          <span class="ml-2 badge badge-sm badge-neutral">env</span>
        </div>
      </th>
      <th class="w-full">Description</th>
      <th>
        Status
        <ColumnFilter
          :options="statusFilterOptions"
          :model-value="filters.status"
          @update:model-value="onFilterChange('status', $event)"
        />
      </th>
      <th>Repository</th>
      <th></th>
    </template>

    <template #row="{ item }: { item: Service }">
      <td class="font-medium name-env">
        {{ item.name }}
        <EnvironmentBadge :environment="item.environment" />
      </td>
      <td class="max-w-md truncate">{{ item.description || '-' }}</td>

      <td><StatusBadge :status="item.status" /></td>
      <td>
        <a
          v-if="item.repository_url"
          :href="item.repository_url"
          target="_blank"
          class="link link-primary"
          @click.stop
          >Link</a
        >
        <span v-else>-</span>
      </td>
      <td>
        <button
          class="group cursor-pointer"
          title="Duplicate"
          @click.stop="handleDuplicate(item.id)"
        >
          <Copy class="w-4 h-4 group-hover:stroke-3 transition-all" />
        </button>
      </td>
    </template>

    <template #form="{ onSubmit, onCancel }">
      <ServiceForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>

  <!-- Duplicate Modal -->
  <Modal
    title="Duplicate Service"
    :open="showDuplicateModal"
    @close="showDuplicateModal = false"
  >
    <ServiceForm
      v-if="duplicateSource"
      :duplicate-service="duplicateSource"
      @submit="handleDuplicateSubmit"
      @cancel="showDuplicateModal = false"
    />
  </Modal>
</template>
