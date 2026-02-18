<script setup lang="ts">
import { computed, type Component } from 'vue';
import type { LinkStep, EntityListItem } from '@/types/relations';
import type { PaginatedResponse } from '@/types';
import Modal from './Modal.vue';
import EntitySelector from './EntitySelector.vue';

const props = defineProps<{
  /** Whether the modal is open */
  open: boolean;
  /** Current step in the link flow */
  step: LinkStep;
  /** Title for the modal (varies by step) */
  title: string;
  /** Singular title for create step (e.g., "Infra", "Service") */
  singularTitle: string;
  /** Selected entity (set after selection or creation) */
  selectedEntity: { id: string; name: string } | null;
  /** Initial name for create form (from search input) */
  initialName: string;
  /** API function to fetch available entities */
  fetchFn: (params: { search?: string }) => Promise<PaginatedResponse<EntityListItem>>;
  /** IDs to exclude from entity selector */
  excludeIds?: string[];
  /** Whether entity creation is allowed */
  allowCreate?: boolean;
  /** Component for creating new entities */
  createFormComponent: Component;
  /** Component for link metadata form (null if no form step) */
  linkFormComponent: Component | null;
  /** Additional props to pass to create form */
  createFormProps?: Record<string, unknown>;
  /** Additional props to pass to link form */
  linkFormProps?: Record<string, unknown>;
  /** Whether this is create-only (no entity selection) */
  createOnly?: boolean;
}>();

const emit = defineEmits<{
  /** Modal close requested */
  close: [];
  /** Entity selected from list */
  select: [entity: { id: string; name: string }];
  /** Create new entity requested (from selector) */
  createRequest: [searchTerm: string];
  /** Entity created (form submitted) */
  create: [data: unknown];
  /** Link submitted (form submitted) */
  link: [data: unknown];
  /** Back to select step requested */
  backToSelect: [];
}>();

const modalTitle = computed(() => {
  if (props.createOnly) {
    return `Create ${props.singularTitle}`;
  }
  if (props.step === 'create') {
    return `Create ${props.singularTitle}`;
  }
  return `Link ${props.singularTitle}`;
});
</script>

<template>
  <Modal :title="modalTitle" :open="open" @close="emit('close')">
    <!-- Create-only mode: show create form directly -->
    <component
      v-if="createOnly"
      :is="createFormComponent"
      :initial-name="initialName"
      v-bind="createFormProps"
      @submit="(data: unknown) => emit('create', data)"
      @cancel="emit('close')"
    />

    <!-- Normal flow: select step -->
    <EntitySelector
      v-else-if="step === 'select'"
      :title="title"
      :fetch-fn="fetchFn"
      :exclude-ids="excludeIds"
      :allow-create="allowCreate"
      @select="(entity) => emit('select', entity)"
      @create="(term) => emit('createRequest', term)"
      @cancel="emit('close')"
    />

    <!-- Create step -->
    <component
      v-else-if="step === 'create'"
      :is="createFormComponent"
      :initial-name="initialName"
      v-bind="createFormProps"
      @submit="(data: unknown) => emit('create', data)"
      @cancel="emit('backToSelect')"
    />

    <!-- Link form step -->
    <component
      v-else-if="step === 'form' && linkFormComponent && selectedEntity"
      :is="linkFormComponent"
      v-bind="linkFormProps"
      @submit="(data: unknown) => emit('link', data)"
      @cancel="emit('close')"
    />
  </Modal>
</template>
