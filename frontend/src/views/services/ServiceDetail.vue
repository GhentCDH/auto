<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { servicesApi, healthchecksApi } from '@/api';
import type {
  ServiceWithRelations,
  LinkInfra,
  CreateHealthcheck,
  InfraRelation,
  UpdateHealthcheck,
} from '@/types';
import { createServiceRelationConfigs } from '@/config/relationConfigs';
import { useRelationManager } from '@/composables/useRelationManager';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import RelationCard from '@/components/common/RelationCard.vue';
import RelationLinkModal from '@/components/common/RelationLinkModal.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import LinkInfraForm from '@/components/forms/LinkInfraForm.vue';
import { infraTypes } from '@/values';

const route = useRoute();
const router = useRouter();

const service = ref<ServiceWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

// Create relation configs with a getter function for service
const relationConfigs = createServiceRelationConfigs(() => service.value, id);

// Initialize relation manager
const {
  modalStates,
  unlinkState,
  unlinkMessage,
  openLinkModal,
  closeLinkModal,
  setLinkStep,
  handleEntitySelect,
  handleCreateRequest,
  handleCreate,
  handleLink,
  openEditModal,
  closeEditModal,
  handleEdit,
  confirmUnlink,
  handleUnlink,
  cancelUnlink,
} = useRelationManager(relationConfigs, {
  onSuccess: () => loadData(),
  onError: (msg) => (error.value = msg),
});

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    service.value = await servicesApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load service';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await servicesApi.update(
      id,
      formData as Parameters<typeof servicesApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update service';
  }
}

async function handleDelete() {
  try {
    await servicesApi.delete(id);
    router.push('/services');
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete service';
  }
}

// Helper to get config
const getConfig = (type: string) => relationConfigs[type as keyof typeof relationConfigs];

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="service">
      <!-- Header -->
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li>
                <router-link to="/services">Services</router-link>
              </li>
              <li>{{ service.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ service.name }}
            <EnvironmentBadge :environment="service.environment" />
            <StatusBadge :status="service.status" />
          </h1>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-sm" @click="showEditModal = true">Edit</button>
          <button class="btn btn-sm btn-error" @click="showDeleteDialog = true">
            Delete
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div class="lg:col-span-2 space-y-6">
          <!-- Details Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Details</h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <div class="text-sm text-base-content/70">Description</div>
                  <div>{{ service.description || '-' }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Repository</div>
                  <a
                    v-if="service.repository_url"
                    :href="service.repository_url"
                    target="_blank"
                    class="link link-primary"
                    >{{ service.repository_url }}</a
                  >
                  <span v-else>-</span>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Created</div>
                  <div>{{ new Date(service.created_at).toLocaleString() }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Updated</div>
                  <div>{{ new Date(service.updated_at).toLocaleString() }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Infrastructure Card -->
          <RelationCard
            :title="getConfig('infra')!.title"
            :items="service.infra"
            :empty-message="getConfig('infra')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            :table-layout="true"
            @add="openLinkModal('infra')"
            @edit="(i) => openEditModal('infra', i)"
            @unlink="(i) => confirmUnlink('infra', i.id, i.name)"
          >
            <template #tableHeader>
              <th>Name</th>
              <th>Type</th>
              <th>Notes</th>
            </template>
            <template #tableRow="{ item: i }">
              <td
                class="cursor-pointer hover:text-primary"
                @click="router.push(`/infra/${i.id}`)"
              >
                {{ i.name }}
              </td>
              <td>
                {{ infraTypes[i.type as keyof typeof infraTypes] || i.type }}
              </td>
              <td>{{ i.relation_notes || '-' }}</td>
            </template>
          </RelationCard>

          <!-- Healthchecks Card -->
          <RelationCard
            :title="getConfig('health')!.title"
            :items="service.healthchecks"
            :empty-message="getConfig('health')!.emptyMessage"
            :show-edit="false"
            :show-unlink="false"
            @add="openLinkModal('health')"
          >
            <template #listItem="{ item: h }">
              <div
                class="flex items-center justify-between cursor-pointer hover:bg-base-300 rounded p-1 -m-1"
                @click="router.push(`/healthchecks/${h.id}`)"
              >
                <div>
                  <span class="font-medium">{{ h.name }}</span>
                  <div class="text-xs text-base-content/70 font-mono">
                    {{ h.protocol }}://{{ h.domain_fqdn }}{{ h.path }}
                  </div>
                </div>
                <StatusBadge :status="h.is_enabled ? 'active' : 'inactive'" />
              </div>
            </template>
          </RelationCard>
        </div>

        <div class="space-y-6">
          <!-- Applications Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Used by Applications</h2>
              <div
                v-if="service.applications.length === 0"
                class="text-base-content/70"
              >
                No applications using this service
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="a in service.applications"
                  :key="a.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/applications/${a.id}`"
                      class="link link-hover"
                    >
                      {{ a.name }}
                    </router-link>
                    <EnvironmentBadge
                      :environment="a.environment"
                      class="ml-2"
                    />
                  </div>
                  <StatusBadge :status="a.status" />
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Service Modal -->
    <Modal
      title="Edit Service"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <ServiceForm
        v-if="service"
        :service="service"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Service"
      message="Are you sure you want to delete this service? This action cannot be undone."
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />

    <!-- Link Infra Modal -->
    <RelationLinkModal
      :open="modalStates.infra.isLinkOpen"
      :step="modalStates.infra.linkStep"
      :title="getConfig('infra')!.title"
      :singular-title="getConfig('infra')!.singularTitle"
      :selected-entity="modalStates.infra.selectedEntity"
      :initial-name="modalStates.infra.initialName"
      :fetch-fn="getConfig('infra')!.listApi"
      :exclude-ids="getConfig('infra')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('infra')!.createFormComponent"
      :link-form-component="getConfig('infra')!.linkFormComponent"
      @close="closeLinkModal('infra')"
      @select="(e) => handleEntitySelect('infra', e)"
      @create-request="(t) => handleCreateRequest('infra', t)"
      @create="(d) => handleCreate('infra', d)"
      @link="(d) => handleLink('infra', d as LinkInfra)"
      @back-to-select="setLinkStep('infra', 'select')"
    />

    <!-- Link/Create Healthcheck Modal -->
    <RelationLinkModal
      :open="modalStates.health.isLinkOpen"
      :step="modalStates.health.linkStep"
      :title="getConfig('health')!.title"
      :singular-title="getConfig('health')!.singularTitle"
      :selected-entity="null"
      :initial-name="modalStates.health.initialName"
      :fetch-fn="getConfig('health')!.listApi"
      :allow-create="true"
      :create-form-component="getConfig('health')!.createFormComponent"
      :link-form-component="null"
      :create-only="true"
      :create-form-props="getConfig('health')!.getCreateFormProps?.()"
      @close="closeLinkModal('health')"
      @create="(d) => handleCreate('health', d)"
    />

    <!-- Unlink Confirmation -->
    <ConfirmDialog
      :open="unlinkState.isOpen"
      title="Unlink Entity"
      :message="unlinkMessage"
      confirm-label="Unlink"
      danger
      @confirm="handleUnlink"
      @cancel="cancelUnlink"
    />

    <!-- Edit Infra Modal -->
    <Modal
      title="Edit Infra Link"
      :open="modalStates.infra.isEditOpen"
      @close="closeEditModal('infra')"
    >
      <LinkInfraForm
        v-if="modalStates.infra.editing"
        @submit="(d) => handleEdit('infra', d as LinkInfra)"
        @cancel="closeEditModal('infra')"
      />
    </Modal>
  </div>
</template>
