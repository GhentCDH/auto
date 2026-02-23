<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { applicationsApi, notesApi } from '@/api';
import type {
  Application,
  ApplicationWithRelations,
  CreateApplication,
  UpdateApplication,
} from '@/types';
import EntityList from '@/components/common/EntityList.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';
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
  console.log(key, value);
}

// Duplicate state
const duplicateSource = ref<ApplicationWithRelations | null>(null);
const showDuplicateModal = ref(false);
const duplicating = ref(false);

async function handleDuplicate(id: string) {
  try {
    duplicateSource.value = await applicationsApi.get(id);
    showDuplicateModal.value = true;
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to load application');
  }
}

async function handleDuplicateSubmit(
  formData: CreateApplication | UpdateApplication
) {
  if (!duplicateSource.value) return;
  duplicating.value = true;

  try {
    const newApp = await applicationsApi.create(formData as CreateApplication);
    const source = duplicateSource.value;

    // Copy all relations in parallel (except healthchecks)
    const relationPromises: Promise<unknown>[] = [];

    for (const svc of source.services) {
      relationPromises.push(
        applicationsApi.linkService(newApp.id, svc.id, {
          notes: svc.relation_notes || undefined,
        })
      );
    }
    for (const infra of source.infra) {
      relationPromises.push(
        applicationsApi.linkInfra(newApp.id, infra.id, {
          notes: infra.relation_notes || undefined,
        })
      );
    }
    for (const domain of source.domains) {
      relationPromises.push(
        applicationsApi.linkDomain(newApp.id, domain.id, {
          notes: domain.relation_notes || undefined,
        })
      );
    }
    for (const person of source.people) {
      relationPromises.push(
        applicationsApi.linkPerson(newApp.id, person.id, {
          contribution_type: person.contribution_type || undefined,
          start_date: person.start_date || undefined,
          end_date: person.end_date || undefined,
          notes: person.relation_notes || undefined,
        })
      );
    }
    for (const share of source.network_shares) {
      relationPromises.push(
        applicationsApi.linkShare(newApp.id, share.id, {
          usage: share.usage || undefined,
          mount_point: share.mount_point || undefined,
          permissions: share.permissions || undefined,
          notes: share.relation_notes || undefined,
        })
      );
    }
    for (const stack of source.stacks) {
      relationPromises.push(applicationsApi.linkStack(newApp.id, stack.id));
    }
    for (const note of source.notes) {
      relationPromises.push(
        notesApi.create({
          entity_type: 'application',
          entity_id: newApp.id,
          title: note.title,
          content: note.content || undefined,
          note_type: note.note_type || undefined,
          url: note.url || undefined,
          is_pinned: note.is_pinned,
        })
      );
    }

    await Promise.allSettled(relationPromises);

    showDuplicateModal.value = false;
    duplicateSource.value = null;
    toast.success('Application duplicated successfully');
    router.push(`/applications/${newApp.id}`);
  } catch (e: unknown) {
    toast.error(
      e instanceof Error ? e.message : 'Failed to duplicate application'
    );
  } finally {
    duplicating.value = false;
  }
}
</script>

<template>
  <EntityList
    ref="entityListRef"
    title="Applications"
    add-label="Add Application"
    search-placeholder="Search applications..."
    empty-message="No applications found"
    modal-title="Create Application"
    base-path="/applications"
    :fetch-fn="applicationsApi.list"
    :create-fn="applicationsApi.create"
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
      <th class="w-full max-w-md">Description</th>
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

    <template #row="{ item }: { item: Application }">
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
      <ApplicationForm @submit="onSubmit" @cancel="onCancel" />
    </template>
  </EntityList>

  <!-- Duplicate Modal -->
  <Modal
    title="Duplicate Application"
    :open="showDuplicateModal"
    @close="showDuplicateModal = false"
  >
    <ApplicationForm
      v-if="duplicateSource"
      :duplicate-application="duplicateSource"
      @submit="handleDuplicateSubmit"
      @cancel="showDuplicateModal = false"
    />
  </Modal>
</template>
