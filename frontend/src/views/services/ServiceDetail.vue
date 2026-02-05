<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { servicesApi, infraApi } from '@/api';
import type {
  ServiceWithRelations,
  LinkInfra,
  CreateInfra,
  InfraRelation,
} from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import EntitySelector from '@/components/common/EntitySelector.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import LinkInfraForm from '@/components/forms/LinkInfraForm.vue';
import { infraTypes } from '@/values';
import { Plus, Edit, Link2Off } from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();

const service = ref<ServiceWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

// Link modal states
type LinkStep = 'select' | 'create' | 'form';
const linkStep = ref<LinkStep>('select');
const selectedEntity = ref<{ id: string; name: string } | null>(null);
const initialName = ref('');
const showLinkInfraModal = ref(false);

// Edit modal states
const showEditInfraModal = ref(false);
const editingInfra = ref<InfraRelation | null>(null);

// Unlink confirm states
const unlinkType = ref<string>('');
const unlinkId = ref<string>('');
const unlinkName = ref<string>('');
const showUnlinkDialog = ref(false);

const id = route.params.id as string;

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

// Link handlers
function openLinkInfraModal() {
  linkStep.value = 'select';
  selectedEntity.value = null;
  initialName.value = '';
  showLinkInfraModal.value = true;
}

function closeLinkInfraModal() {
  showLinkInfraModal.value = false;
}

function handleEntitySelect(entity: { id: string; name: string }) {
  selectedEntity.value = entity;
  linkStep.value = 'form';
}

function handleCreateRequest(searchTerm: string) {
  initialName.value = searchTerm;
  linkStep.value = 'create';
}

async function handleCreateInfra(data: CreateInfra) {
  try {
    const created = await infraApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create infra';
  }
}

async function handleLinkInfra(data: LinkInfra) {
  if (!selectedEntity.value) return;
  try {
    await servicesApi.linkInfra(id, selectedEntity.value.id, data);
    showLinkInfraModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link infra';
  }
}

// Edit handler
function openEditInfra(infra: InfraRelation) {
  editingInfra.value = infra;
  showEditInfraModal.value = true;
}

async function handleEditInfra(data: LinkInfra) {
  if (!editingInfra.value) return;
  try {
    await servicesApi.linkInfra(id, editingInfra.value.id, data);
    showEditInfraModal.value = false;
    editingInfra.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update infra link';
  }
}

// Unlink handlers
function confirmUnlink(type: string, entityId: string, name: string) {
  unlinkType.value = type;
  unlinkId.value = entityId;
  unlinkName.value = name;
  showUnlinkDialog.value = true;
}

async function handleUnlink() {
  try {
    if (unlinkType.value === 'infra') {
      await servicesApi.unlinkInfra(id, unlinkId.value);
    }
    showUnlinkDialog.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to unlink';
  }
}

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="service">
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

          <!-- Infra Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Infrastructure</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkInfraModal"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div
                v-if="service.infra.length === 0"
                class="text-base-content/70"
              >
                No infrastructure linked
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Type</th>
                      <th>Notes</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="i in service.infra" :key="i.id">
                      <td
                        class="cursor-pointer hover:text-primary"
                        @click="router.push(`/infra/${i.id}`)"
                      >
                        {{ i.name }}
                      </td>
                      <td>
                        {{
                          infraTypes[i.type as keyof typeof infraTypes] ||
                          i.type
                        }}
                      </td>
                      <td>{{ i.relation_notes || '-' }}</td>
                      <td class="flex">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click="openEditInfra(i)"
                        >
                          <Edit class="w-4 h-4" />
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click="confirmUnlink('infra', i.id, i.name)"
                        >
                          <Link2Off class="w-4 h-4" />
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
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

          <!-- Healthchecks Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Healthchecks</h2>
                <router-link
                  :to="`/healthchecks?service_id=${id}`"
                  class="btn btn-sm btn-ghost"
                >
                  View All
                </router-link>
              </div>
              <div
                v-if="service.healthchecks.length === 0"
                class="text-base-content/70"
              >
                No healthchecks configured
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="h in service.healthchecks"
                  :key="h.id"
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
    <Modal
      :title="linkStep === 'create' ? 'Create Infra' : 'Link Infra'"
      :open="showLinkInfraModal"
      @close="closeLinkInfraModal"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Infrastructure"
        :fetch-fn="infraApi.list"
        :exclude-ids="service?.infra.map((i) => i.id)"
        :allow-create="true"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkInfraModal"
      />
      <InfraForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateInfra(data as CreateInfra)"
        @cancel="linkStep = 'select'"
      />
      <LinkInfraForm
        v-else-if="selectedEntity"
        @submit="handleLinkInfra"
        @cancel="closeLinkInfraModal"
      />
    </Modal>

    <!-- Unlink Confirmation -->
    <ConfirmDialog
      :open="showUnlinkDialog"
      title="Unlink Entity"
      :message="`Are you sure you want to unlink '${unlinkName}' from this service?`"
      confirm-label="Unlink"
      danger
      @confirm="handleUnlink"
      @cancel="showUnlinkDialog = false"
    />

    <!-- Edit Infra Modal -->
    <Modal
      title="Edit Infra Link"
      :open="showEditInfraModal"
      @close="showEditInfraModal = false"
    >
      <LinkInfraForm
        v-if="editingInfra"
        @submit="handleEditInfra"
        @cancel="showEditInfraModal = false"
      />
    </Modal>
  </div>
</template>
