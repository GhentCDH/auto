<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { applicationsApi, notesApi } from '@/api';
import type {
  ApplicationWithRelations,
  CreateNote,
  Note,
  UpdateNote,
  LinkInfra,
  LinkService,
  LinkDomain,
  LinkPerson,
  LinkNetworkShare,
  UpdateHealthcheck,
  InfraRelation,
  ServiceRelation,
  DomainRelation,
  PersonRelation,
  NetworkShareRelation,
  HealthcheckRelation,
} from '@/types';
import { createApplicationRelationConfigs } from '@/config/relationConfigs';
import { useRelationManager } from '@/composables/useRelationManager';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import RelationCard from '@/components/common/RelationCard.vue';
import RelationLinkModal from '@/components/common/RelationLinkModal.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';
import LinkInfraForm from '@/components/forms/LinkInfraForm.vue';
import LinkServiceForm from '@/components/forms/LinkServiceForm.vue';
import LinkDomainForm from '@/components/forms/LinkDomainForm.vue';
import LinkPersonForm from '@/components/forms/LinkPersonForm.vue';
import LinkShareForm from '@/components/forms/LinkShareForm.vue';
import NoteForm from '@/components/forms/NoteForm.vue';
import MarkdownRenderer from '@/components/common/MarkdownRenderer.vue';
import StackBadge from '@/components/common/StackBadge.vue';
import { infraTypes } from '@/values';
import { Pin, ExternalLink, Edit, Link2Off } from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();

const app = ref<ApplicationWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const id = route.params.id as string;

// Create relation configs with a getter function for app
const relationConfigs = createApplicationRelationConfigs(() => app.value, id);

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

// Note modal states (Notes use a different CRUD pattern)
const showCreateNoteModal = ref(false);
const showEditNoteModal = ref(false);
const showViewNoteModal = ref(false);
const showDeleteNoteDialog = ref(false);
const editingNote = ref<Note | null>(null);
const viewingNote = ref<Note | null>(null);
const deletingNote = ref<Note | null>(null);

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    app.value = await applicationsApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load application';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await applicationsApi.update(
      id,
      formData as Parameters<typeof applicationsApi.update>[1]
    );
    showEditModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update application';
  }
}

async function handleDelete() {
  try {
    await applicationsApi.delete(id);
    router.push('/applications');
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to delete application';
  }
}

// Note handlers (separate CRUD pattern)
function openViewNote(note: Note) {
  viewingNote.value = note;
  showViewNoteModal.value = true;
}

function openEditNote(note: Note) {
  editingNote.value = note;
  showEditNoteModal.value = true;
}

function confirmDeleteNote(note: Note) {
  deletingNote.value = note;
  showDeleteNoteDialog.value = true;
}

async function handleCreateNote(data: CreateNote) {
  try {
    await notesApi.create(data);
    showCreateNoteModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create note';
  }
}

async function handleEditNote(data: UpdateNote) {
  if (!editingNote.value) return;
  try {
    await notesApi.update(editingNote.value.id, data);
    showEditNoteModal.value = false;
    editingNote.value = null;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update note';
  }
}

async function handleDeleteNote() {
  if (!deletingNote.value) return;
  try {
    await notesApi.delete(deletingNote.value.id);
    showDeleteNoteDialog.value = false;
    deletingNote.value = null;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to delete note';
  }
}

// Helper to get config
const getConfig = (type: string) => relationConfigs[type as keyof typeof relationConfigs];

// Computed props for link forms
const domainLinkFormProps = computed(() => ({
  domainName: modalStates.domain.selectedEntity?.name,
}));

const personLinkFormProps = computed(() => ({
  personName: modalStates.person.selectedEntity?.name,
}));

const shareLinkFormProps = computed(() => ({
  shareName: modalStates.share.selectedEntity?.name,
}));

// Edit form props
const editDomainFormProps = computed(() => ({
  domainName: (modalStates.domain.editing as DomainRelation)?.fqdn,
  initial: modalStates.domain.editing,
}));

const editPersonFormProps = computed(() => ({
  personName: (modalStates.person.editing as PersonRelation)?.name,
  initial: modalStates.person.editing,
}));

const editShareFormProps = computed(() => ({
  shareName: (modalStates.share.editing as NetworkShareRelation)?.name,
  initial: modalStates.share.editing,
}));

onMounted(loadData);
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="app">
      <!-- Header -->
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li>
                <router-link to="/applications">Applications</router-link>
              </li>
              <li>{{ app.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ app.name }}
            <EnvironmentBadge :environment="app.environment" />
            <StatusBadge :status="app.status" />
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
                  <div>{{ app.description || '-' }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">URL</div>
                  <a
                    v-if="app.url"
                    :href="app.url"
                    target="_blank"
                    class="link link-primary"
                    >{{ app.url }}</a
                  >
                  <span v-else>-</span>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Repository</div>
                  <a
                    v-if="app.repository_url"
                    :href="app.repository_url"
                    target="_blank"
                    class="link link-primary"
                    >{{ app.repository_url }}</a
                  >
                  <span v-else>-</span>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Created</div>
                  <div>{{ new Date(app.created_at).toLocaleString() }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Services Card -->
          <RelationCard
            :title="getConfig('service')!.title"
            :items="app.services"
            :empty-message="getConfig('service')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            :table-layout="true"
            @add="openLinkModal('service')"
            @edit="(s) => openEditModal('service', s)"
            @unlink="(s) => confirmUnlink('service', s.id, s.name)"
          >
            <template #tableHeader>
              <th class="name-env">
                <div>Name</div>
                <span class="badge badge-sm badge-ghost badge-outline">env</span>
              </th>
              <th class="w-full">Notes</th>
              <th class="w-10">Status</th>
            </template>
            <template #tableRow="{ item: s }">
              <td
                class="cursor-pointer hover:text-primary name-env"
                @click="router.push(`/services/${s.id}`)"
              >
                {{ s.name }}
                <EnvironmentBadge :environment="s.environment" />
              </td>
              <td>{{ s.relation_notes || '-' }}</td>
              <td><StatusBadge :status="s.status" /></td>
            </template>
          </RelationCard>

          <!-- Domains Card -->
          <RelationCard
            :title="getConfig('domain')!.title"
            :items="app.domains"
            :empty-message="getConfig('domain')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            :table-layout="true"
            @add="openLinkModal('domain')"
            @edit="(d) => openEditModal('domain', d)"
            @unlink="(d) => confirmUnlink('domain', d.id, d.fqdn)"
          >
            <template #tableHeader>
              <th>Domain</th>
              <th>Target</th>
            </template>
            <template #tableRow="{ item: d }">
              <td
                class="cursor-pointer hover:text-primary"
                @click="router.push(`/domains/${d.id}`)"
              >
                {{ d.fqdn }}
              </td>
              <td
                v-if="d.target_application_id"
                class="cursor-pointer hover:text-primary hover:underline"
                @click="router.push(`/applications/${d.target_application_id}`)"
              >
                {{ d.target_application_name }}
                {{ d.target_application_name === app.name ? '(this!)' : '' }}
              </td>
              <td
                v-else-if="d.target_service_id"
                class="cursor-pointer hover:text-primary hover:underline"
                @click="router.push(`/services/${d.target_service_id}`)"
              >
                {{ d.target_service_name }}
              </td>
              <td v-else></td>
            </template>
          </RelationCard>

          <!-- Infrastructure Card -->
          <RelationCard
            :title="getConfig('infra')!.title"
            :items="app.infra"
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

          <!-- Network Shares Card -->
          <RelationCard
            :title="getConfig('share')!.title"
            :items="app.network_shares"
            :empty-message="getConfig('share')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            @add="openLinkModal('share')"
            @edit="(s) => openEditModal('share', s)"
            @unlink="(s) => confirmUnlink('share', s.id, s.name)"
          >
            <template #listItem="{ item: s }">
              <router-link :to="`/shares/${s.id}`" class="link link-hover">
                {{ s.name }}
              </router-link>
              <div class="text-xs text-base-content/70">{{ s.path }}</div>
            </template>
          </RelationCard>

          <!-- Healthchecks Card -->
          <RelationCard
            :title="getConfig('health')!.title"
            :items="app.healthchecks"
            :empty-message="getConfig('health')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            :table-layout="true"
            @add="openLinkModal('health')"
            @edit="(h) => openEditModal('health', h)"
            @unlink="(h) => confirmUnlink('health', h.id, h.name)"
          >
            <template #tableHeader>
              <th>Name</th>
              <th>URL</th>
              <th>Expected</th>
              <th class="w-10">Status</th>
            </template>
            <template #tableRow="{ item: h }">
              <td
                class="cursor-pointer hover:text-primary"
                @click="router.push(`/healthchecks/${h.id}`)"
              >
                {{ h.name }}
              </td>
              <td class="font-mono text-xs truncate max-w-50">
                {{ h.protocol }}://{{ h.domain_fqdn }}{{ h.path }}
              </td>
              <td>{{ h.expected_status }}</td>
              <td>
                <StatusBadge :status="h.is_enabled ? 'active' : 'inactive'" />
              </td>
            </template>
          </RelationCard>
        </div>

        <div class="space-y-6">
          <!-- People Card -->
          <RelationCard
            :title="getConfig('person')!.title"
            :items="app.people"
            :empty-message="getConfig('person')!.emptyMessage"
            :show-edit="true"
            :show-unlink="true"
            @add="openLinkModal('person')"
            @edit="(p) => openEditModal('person', p)"
            @unlink="(p) => confirmUnlink('person', p.id, p.name)"
          >
            <template #listItem="{ item: p }">
              <router-link :to="`/people/${p.id}`" class="link link-hover">
                {{ p.name }}
              </router-link>
              <span class="badge badge-outline badge-sm ml-2">
                {{ p.contribution_type }}
              </span>
            </template>
          </RelationCard>

          <!-- Tech Stack Card -->
          <RelationCard
            :title="getConfig('stack')!.title"
            :items="app.stacks"
            :empty-message="getConfig('stack')!.emptyMessage"
            :show-edit="false"
            :show-unlink="false"
            @add="openLinkModal('stack')"
          >
            <template #badge="{ item: s }">
              <StackBadge
                :name="s.name"
                removable
                clickable
                @click="router.push(`/stack/${s.id}`)"
                @remove="confirmUnlink('stack', s.id, s.name)"
              />
            </template>
          </RelationCard>

          <!-- Notes Card (separate pattern) -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Notes</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="showCreateNoteModal = true"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                  Add
                </button>
              </div>
              <div v-if="app.notes.length === 0" class="text-base-content/70">
                No notes
              </div>
              <ul v-else class="space-y-3">
                <li
                  v-for="n in app.notes"
                  :key="n.id"
                  class="border-b border-base-300 pb-2 last:border-0"
                >
                  <div class="flex items-start justify-between gap-2">
                    <div
                      class="flex-1 min-w-0 cursor-pointer hover:bg-base-300/50 rounded p-1 -m-1 transition-colors"
                      @click="openViewNote(n)"
                    >
                      <div class="flex items-center gap-2">
                        <Pin
                          v-if="n.is_pinned"
                          class="h-3 w-3 text-primary shrink-0"
                        />
                        <span class="font-medium hover:text-primary truncate">
                          {{ n.title }}
                        </span>
                        <span class="badge badge-xs badge-ghost">{{
                          n.note_type
                        }}</span>
                      </div>
                      <div
                        v-if="n.content"
                        class="text-sm text-base-content/70 line-clamp-3 mt-1 overflow-hidden"
                      >
                        <MarkdownRenderer :content="n.content" />
                      </div>
                      <div
                        v-if="n.url"
                        class="text-sm text-primary inline-flex items-center mt-1"
                      >
                        <ExternalLink class="h-3 w-3" />
                        Link attached
                      </div>
                    </div>
                    <div class="flex shrink-0 justify-end">
                      <button
                        class="btn btn-ghost btn-xs"
                        @click.stop="openEditNote(n)"
                      >
                        <Edit class="w-4 h-4" />
                      </button>
                      <button
                        class="btn btn-ghost btn-xs text-error"
                        @click.stop="confirmDeleteNote(n)"
                      >
                        <Link2Off class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Application Modal -->
    <Modal
      title="Edit Application"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <ApplicationForm
        v-if="app"
        :application="app"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Application"
      message="Are you sure you want to delete this application? This action cannot be undone."
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

    <!-- Link Service Modal -->
    <RelationLinkModal
      :open="modalStates.service.isLinkOpen"
      :step="modalStates.service.linkStep"
      :title="getConfig('service')!.title"
      :singular-title="getConfig('service')!.singularTitle"
      :selected-entity="modalStates.service.selectedEntity"
      :initial-name="modalStates.service.initialName"
      :fetch-fn="getConfig('service')!.listApi"
      :exclude-ids="getConfig('service')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('service')!.createFormComponent"
      :link-form-component="getConfig('service')!.linkFormComponent"
      @close="closeLinkModal('service')"
      @select="(e) => handleEntitySelect('service', e)"
      @create-request="(t) => handleCreateRequest('service', t)"
      @create="(d) => handleCreate('service', d)"
      @link="(d) => handleLink('service', d as LinkService)"
      @back-to-select="setLinkStep('service', 'select')"
    />

    <!-- Link Domain Modal -->
    <RelationLinkModal
      :open="modalStates.domain.isLinkOpen"
      :step="modalStates.domain.linkStep"
      :title="getConfig('domain')!.title"
      :singular-title="getConfig('domain')!.singularTitle"
      :selected-entity="modalStates.domain.selectedEntity"
      :initial-name="modalStates.domain.initialName"
      :fetch-fn="getConfig('domain')!.listApi"
      :exclude-ids="getConfig('domain')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('domain')!.createFormComponent"
      :link-form-component="getConfig('domain')!.linkFormComponent"
      :link-form-props="domainLinkFormProps"
      @close="closeLinkModal('domain')"
      @select="(e) => handleEntitySelect('domain', e)"
      @create-request="(t) => handleCreateRequest('domain', t)"
      @create="(d) => handleCreate('domain', d)"
      @link="(d) => handleLink('domain', d as LinkDomain)"
      @back-to-select="setLinkStep('domain', 'select')"
    />

    <!-- Link Person Modal -->
    <RelationLinkModal
      :open="modalStates.person.isLinkOpen"
      :step="modalStates.person.linkStep"
      :title="getConfig('person')!.title"
      :singular-title="getConfig('person')!.singularTitle"
      :selected-entity="modalStates.person.selectedEntity"
      :initial-name="modalStates.person.initialName"
      :fetch-fn="getConfig('person')!.listApi"
      :exclude-ids="getConfig('person')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('person')!.createFormComponent"
      :link-form-component="getConfig('person')!.linkFormComponent"
      :link-form-props="personLinkFormProps"
      @close="closeLinkModal('person')"
      @select="(e) => handleEntitySelect('person', e)"
      @create-request="(t) => handleCreateRequest('person', t)"
      @create="(d) => handleCreate('person', d)"
      @link="(d) => handleLink('person', d as LinkPerson)"
      @back-to-select="setLinkStep('person', 'select')"
    />

    <!-- Link Share Modal -->
    <RelationLinkModal
      :open="modalStates.share.isLinkOpen"
      :step="modalStates.share.linkStep"
      :title="getConfig('share')!.title"
      :singular-title="getConfig('share')!.singularTitle"
      :selected-entity="modalStates.share.selectedEntity"
      :initial-name="modalStates.share.initialName"
      :fetch-fn="getConfig('share')!.listApi"
      :exclude-ids="getConfig('share')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('share')!.createFormComponent"
      :link-form-component="getConfig('share')!.linkFormComponent"
      :link-form-props="shareLinkFormProps"
      @close="closeLinkModal('share')"
      @select="(e) => handleEntitySelect('share', e)"
      @create-request="(t) => handleCreateRequest('share', t)"
      @create="(d) => handleCreate('share', d)"
      @link="(d) => handleLink('share', d as LinkNetworkShare)"
      @back-to-select="setLinkStep('share', 'select')"
    />

    <!-- Link Stack Modal -->
    <RelationLinkModal
      :open="modalStates.stack.isLinkOpen"
      :step="modalStates.stack.linkStep"
      :title="getConfig('stack')!.title"
      :singular-title="getConfig('stack')!.singularTitle"
      :selected-entity="modalStates.stack.selectedEntity"
      :initial-name="modalStates.stack.initialName"
      :fetch-fn="getConfig('stack')!.listApi"
      :exclude-ids="getConfig('stack')!.getExcludeIds?.()"
      :allow-create="true"
      :create-form-component="getConfig('stack')!.createFormComponent"
      :link-form-component="null"
      @close="closeLinkModal('stack')"
      @select="(e) => handleEntitySelect('stack', e)"
      @create-request="(t) => handleCreateRequest('stack', t)"
      @create="(d) => handleCreate('stack', d)"
      @back-to-select="setLinkStep('stack', 'select')"
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

    <!-- Edit Service Modal -->
    <Modal
      title="Edit Service Link"
      :open="modalStates.service.isEditOpen"
      @close="closeEditModal('service')"
    >
      <LinkServiceForm
        v-if="modalStates.service.editing"
        @submit="(d) => handleEdit('service', d as LinkService)"
        @cancel="closeEditModal('service')"
      />
    </Modal>

    <!-- Edit Domain Modal -->
    <Modal
      title="Edit Domain Link"
      :open="modalStates.domain.isEditOpen"
      @close="closeEditModal('domain')"
    >
      <LinkDomainForm
        v-if="modalStates.domain.editing"
        :domain-name="(modalStates.domain.editing as DomainRelation).fqdn"
        :initial="modalStates.domain.editing as DomainRelation"
        @submit="(d) => handleEdit('domain', d as LinkDomain)"
        @cancel="closeEditModal('domain')"
      />
    </Modal>

    <!-- Edit Person Modal -->
    <Modal
      title="Edit Person Link"
      :open="modalStates.person.isEditOpen"
      @close="closeEditModal('person')"
    >
      <LinkPersonForm
        v-if="modalStates.person.editing"
        :person-name="(modalStates.person.editing as PersonRelation).name"
        :initial="modalStates.person.editing as PersonRelation"
        @submit="(d) => handleEdit('person', d as LinkPerson)"
        @cancel="closeEditModal('person')"
      />
    </Modal>

    <!-- Edit Share Modal -->
    <Modal
      title="Edit Storage Link"
      :open="modalStates.share.isEditOpen"
      @close="closeEditModal('share')"
    >
      <LinkShareForm
        v-if="modalStates.share.editing"
        :share-name="(modalStates.share.editing as NetworkShareRelation).name"
        :initial="modalStates.share.editing as NetworkShareRelation"
        @submit="(d) => handleEdit('share', d as LinkNetworkShare)"
        @cancel="closeEditModal('share')"
      />
    </Modal>

    <!-- Edit Healthcheck Modal -->
    <Modal
      title="Edit Healthcheck"
      :open="modalStates.health.isEditOpen"
      @close="closeEditModal('health')"
    >
      <component
        v-if="modalStates.health.editing"
        :is="getConfig('health')!.createFormComponent"
        :healthcheck="modalStates.health.editing"
        @submit="(d: unknown) => handleEdit('health', d as UpdateHealthcheck)"
        @cancel="closeEditModal('health')"
      />
    </Modal>

    <!-- Note Modals (separate CRUD pattern) -->
    <Modal
      title="Create Note"
      :open="showCreateNoteModal"
      @close="showCreateNoteModal = false"
    >
      <NoteForm
        entity-type="application"
        :entity-id="id"
        @submit="(data) => handleCreateNote(data as CreateNote)"
        @cancel="showCreateNoteModal = false"
      />
    </Modal>

    <Modal
      title="Edit Note"
      :open="showEditNoteModal"
      @close="showEditNoteModal = false"
    >
      <NoteForm
        v-if="editingNote"
        :note="editingNote"
        entity-type="application"
        :entity-id="id"
        @submit="handleEditNote"
        @cancel="showEditNoteModal = false"
      />
    </Modal>

    <Modal
      :title="viewingNote?.title || 'Note'"
      :open="showViewNoteModal"
      @close="showViewNoteModal = false"
    >
      <div v-if="viewingNote" class="space-y-4">
        <div class="flex items-center gap-2 text-sm text-base-content/70">
          <span class="badge badge-sm">{{ viewingNote.note_type }}</span>
          <span
            v-if="viewingNote.is_pinned"
            class="badge badge-sm badge-primary"
            >Pinned</span
          >
        </div>
        <div v-if="viewingNote.content" class="bg-base-100 rounded-lg p-4">
          <MarkdownRenderer :content="viewingNote.content" />
        </div>
        <div v-else class="text-base-content/50 italic">No content</div>
        <a
          v-if="viewingNote.url"
          :href="viewingNote.url"
          target="_blank"
          class="link link-primary inline-flex items-center"
        >
          <ExternalLink class="h-4 w-4" />
          {{ viewingNote.url }}
        </a>
        <div class="text-xs text-base-content/50 pt-2 border-t border-base-300">
          Created: {{ new Date(viewingNote.created_at).toLocaleString() }}
          <span v-if="viewingNote.updated_at !== viewingNote.created_at">
            | Updated: {{ new Date(viewingNote.updated_at).toLocaleString() }}
          </span>
        </div>
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost" @click="showViewNoteModal = false">
            Close
          </button>
          <button
            class="btn btn-primary"
            @click="
              showViewNoteModal = false;
              openEditNote(viewingNote);
            "
          >
            <Edit class="w-4 h-4" />
          </button>
        </div>
      </div>
    </Modal>

    <ConfirmDialog
      :open="showDeleteNoteDialog"
      title="Delete Note"
      :message="`Are you sure you want to delete '${deletingNote?.title}'? This action cannot be undone.`"
      confirm-label="Delete"
      danger
      @confirm="handleDeleteNote"
      @cancel="showDeleteNoteDialog = false"
    />
  </div>
</template>

<style>
@reference "tailwindcss";

table {
  @apply border-collapse;
}

td {
  @apply border-b-0;
}

tr:not(:last-child) {
  border-bottom: var(--border) solid
    color-mix(in oklch, var(--color-base-content) 5%, #0000);
}
</style>
