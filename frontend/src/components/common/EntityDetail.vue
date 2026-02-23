<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import LoadingSpinner from './LoadingSpinner.vue';
import Modal from './Modal.vue';
import ConfirmDialog from './ConfirmDialog.vue';

type BaseEntity = {
  id: string;
  name: string;
};

const props = defineProps<{
  entityName: string;
  listPath: string;
  listLabel?: string;
  fetchFn: (id: string) => Promise<BaseEntity>;
  updateFn: (id: string, data: unknown) => Promise<unknown>;
  deleteFn: (id: string) => Promise<void>;
}>();

const route = useRoute();
const router = useRouter();

const entity = ref<BaseEntity | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);
const id = route.params.id as string;

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    entity.value = await props.fetchFn(id);
  } catch (e: unknown) {
    error.value =
      e instanceof Error
        ? e.message
        : `Failed to load ${props.entityName.toLowerCase()}`;
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await props.updateFn(id, formData);
    showEditModal.value = false;
    toast.success(`${props.entityName} updated`);
    loadData();
  } catch (e: unknown) {
    toast.error(
      e instanceof Error
        ? e.message
        : `Failed to update ${props.entityName.toLowerCase()}`
    );
  }
}

async function handleDelete() {
  try {
    await props.deleteFn(id);
    toast.success(`${props.entityName} deleted`);
    router.push(props.listPath);
  } catch (e: unknown) {
    toast.error(
      e instanceof Error
        ? e.message
        : `Failed to delete ${props.entityName.toLowerCase()}`
    );
  }
}

function onFormSubmit(formData: unknown) {
  return handleUpdate(formData);
}

onMounted(loadData);

defineExpose({ loadData, entity });

function nameOrFqdn(entity: BaseEntity): string {
  return entity.name;
}
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="entity">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li>
                <router-link :to="listPath">{{
                  listLabel || entityName + 's'
                }}</router-link>
              </li>
              <li>{{ nameOrFqdn(entity) }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            <slot name="title" :entity="entity">
              {{ nameOrFqdn(entity) }}
            </slot>
            <slot name="status" :entity="entity" />
          </h1>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-sm" @click="showEditModal = true">Edit</button>
          <button class="btn btn-sm btn-error" @click="showDeleteDialog = true">
            Delete
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">
              <slot name="details-title">Details</slot>
            </h2>
            <slot name="details" :entity="entity" />
          </div>
        </div>

        <div class="card bg-base-200">
          <div class="card-body">
            <slot name="relations" :entity="entity" />
          </div>
        </div>
      </div>
    </div>

    <Modal
      :title="`Edit ${entityName}`"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <slot
        name="form"
        :entity="entity"
        :on-submit="onFormSubmit"
        :on-cancel="() => (showEditModal = false)"
      />
    </Modal>

    <ConfirmDialog
      :open="showDeleteDialog"
      :title="`Delete ${entityName}`"
      :message="`Are you sure you want to delete this ${entityName.toLowerCase()}?`"
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
