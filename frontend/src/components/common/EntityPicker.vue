<script setup lang="ts">
import { ref, computed, defineAsyncComponent, type Component } from 'vue';
import { toast } from 'vue-sonner';
import EntitySelector from './EntitySelector.vue';
import Modal from './Modal.vue';
import { applicationsApi, servicesApi, infraApi, domainsApi } from '@/api';

type EntityType = 'application' | 'service' | 'infra' | 'domain';

const props = withDefaults(
  defineProps<{
    entityType: EntityType;
    // Display name of the current selection (the parent owns the id in its form).
    selectedName?: string | null;
    badgeClass?: string;
    excludeIds?: string[];
    // Defer creation: instead of POSTing the new entity, emit its form data via
    // `create-deferred` so the parent can persist it (e.g. bundled into its own
    // create request). Used when the new entity's required link is the parent
    // entity that doesn't exist yet — see InfraForm's domain picker.
    deferCreate?: boolean;
    // Extra props forwarded to the create form (e.g. { hideTarget: true }).
    createFormProps?: Record<string, unknown>;
  }>(),
  { selectedName: null, badgeClass: 'badge-primary' }
);

const emit = defineEmits<{
  select: [entity: { id: string; name: string }];
  'create-deferred': [data: unknown];
}>();

// Per-type wiring. Forms are loaded async so EntityPicker can be used *inside*
// the very forms it renders (e.g. DomainForm/InfraForm) without an import cycle.
type Created = { id: string; name?: string; fqdn?: string };

type PickerConfig = {
  title: string;
  badge: string;
  fetchFn: (params: { search?: string }) => Promise<{
    data: Array<{ id: string; name: string; environment?: string }>;
  }>;
  createFn: (data: unknown) => Promise<Created>;
  form: Component;
  nameOf: (created: Created) => string;
};

const configs: Record<EntityType, PickerConfig> = {
  application: {
    title: 'Applications',
    badge: 'App',
    fetchFn: applicationsApi.list,
    createFn: (d) => applicationsApi.create(d as never),
    form: defineAsyncComponent(() => import('../forms/ApplicationForm.vue')),
    nameOf: (c) => c.name ?? '',
  },
  service: {
    title: 'Services',
    badge: 'Service',
    fetchFn: servicesApi.list,
    createFn: (d) => servicesApi.create(d as never),
    form: defineAsyncComponent(() => import('../forms/ServiceForm.vue')),
    nameOf: (c) => c.name ?? '',
  },
  infra: {
    title: 'Infrastructure',
    badge: 'Infra',
    fetchFn: infraApi.list,
    createFn: (d) => infraApi.create(d as never),
    form: defineAsyncComponent(() => import('../forms/InfraForm.vue')),
    nameOf: (c) => c.name ?? '',
  },
  domain: {
    title: 'Domains',
    badge: 'Domain',
    fetchFn: domainsApi.list,
    createFn: (d) => domainsApi.create(d as never),
    form: defineAsyncComponent(() => import('../forms/DomainForm.vue')),
    nameOf: (c) => c.fqdn ?? '',
  },
};

const config = computed(() => configs[props.entityType]);

// Start expanded when nothing is selected yet; otherwise show the summary badge.
const showSelector = ref(!props.selectedName);
const showCreate = ref(false);
const initialName = ref('');

function handleSelect(entity: { id: string; name: string }) {
  emit('select', entity);
  showSelector.value = false;
}

function handleCreateRequest(searchTerm: string) {
  initialName.value = searchTerm;
  showCreate.value = true;
}

async function handleCreated(data: unknown) {
  // Deferred: hand the form data to the parent instead of creating now.
  if (props.deferCreate) {
    emit('create-deferred', data);
    showCreate.value = false;
    showSelector.value = false;
    return;
  }
  try {
    const created = await config.value.createFn(data);
    const name = config.value.nameOf(created);
    toast.success(`${config.value.badge} created`);
    emit('select', { id: created.id, name });
    showCreate.value = false;
    showSelector.value = false;
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to create');
  }
}

function handleCancel() {
  // Keep the selector open if there's nothing to fall back to.
  showSelector.value = !props.selectedName;
}
</script>

<template>
  <div class="bg-base-200 rounded-box p-2">
    <EntitySelector
      v-if="showSelector"
      :title="config.title"
      :fetch-fn="config.fetchFn"
      :exclude-ids="excludeIds"
      allow-create
      @select="handleSelect"
      @create="handleCreateRequest"
      @cancel="handleCancel"
    />
    <div
      v-else-if="selectedName"
      class="flex items-center justify-between px-2 py-1"
    >
      <div class="flex items-center gap-2">
        <span class="badge badge-sm" :class="badgeClass">{{
          config.badge
        }}</span>
        <span class="font-medium">{{ selectedName }}</span>
      </div>
      <button
        type="button"
        class="btn btn-ghost btn-xs"
        @click="showSelector = true"
      >
        Change
      </button>
    </div>
  </div>

  <!-- Create form lives in a teleported Modal so it never nests inside the
       parent form's <form> element (invalid HTML + double submit). -->
  <Modal
    :open="showCreate"
    :title="`Create ${config.badge}`"
    @close="showCreate = false"
  >
    <component
      :is="config.form"
      :initial-name="initialName"
      v-bind="createFormProps"
      @submit="handleCreated"
      @cancel="showCreate = false"
    />
  </Modal>
</template>
