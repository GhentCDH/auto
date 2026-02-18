<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  applicationsApi,
  infraApi,
  servicesApi,
  domainsApi,
  peopleApi,
  sharesApi,
  notesApi,
  stacksApi,
  healthchecksApi,
} from '@/api';
import type {
  ApplicationWithRelations,
  CreateHealthcheck,
  CreateInfra,
  CreateService,
  CreateDomain,
  CreatePerson,
  CreateNetworkShare,
  CreateNote,
  CreateStack,
  DomainRelation,
  Healthcheck,
  InfraRelation,
  LinkInfra,
  LinkService,
  LinkDomain,
  LinkPerson,
  LinkNetworkShare,
  NetworkShareRelation,
  Note,
  PersonRelation,
  UpdateHealthcheck,
  UpdateNote,
  ServiceRelation,
  HealthcheckRelation,
} from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import EntitySelector from '@/components/common/EntitySelector.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
import PersonForm from '@/components/forms/PersonForm.vue';
import ShareForm from '@/components/forms/ShareForm.vue';
import LinkInfraForm from '@/components/forms/LinkInfraForm.vue';
import LinkServiceForm from '@/components/forms/LinkServiceForm.vue';
import LinkDomainForm from '@/components/forms/LinkDomainForm.vue';
import LinkPersonForm from '@/components/forms/LinkPersonForm.vue';
import LinkShareForm from '@/components/forms/LinkShareForm.vue';
import StackForm from '@/components/forms/StackForm.vue';
import NoteForm from '@/components/forms/NoteForm.vue';
import MarkdownRenderer from '@/components/common/MarkdownRenderer.vue';
import StackBadge from '@/components/common/StackBadge.vue';
import { infraTypes } from '@/values';
import { Pin, ExternalLink, Plus, Edit, Link2Off } from 'lucide-vue-next';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';

const route = useRoute();
const router = useRouter();

const app = ref<ApplicationWithRelations | null>(null);
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
const showLinkServiceModal = ref(false);
const showLinkDomainModal = ref(false);
const showLinkPersonModal = ref(false);
const showLinkShareModal = ref(false);
const showLinkStackModal = ref(false);
const showLinkHealthModal = ref(false);

// Edit modal states
const showEditInfraModal = ref(false);
const showEditHealthModal = ref(false);
const showEditServiceModal = ref(false);
const showEditDomainModal = ref(false);
const showEditPersonModal = ref(false);
const showEditShareModal = ref(false);
const editingInfra = ref<InfraRelation | null>(null);
const editingHealth = ref<Healthcheck | null>(null);
const editingService = ref<ServiceRelation | null>(null);
const editingDomain = ref<DomainRelation | null>(null);
const editingPerson = ref<PersonRelation | null>(null);
const editingShare = ref<NetworkShareRelation | null>(null);

// Note modal states
const showCreateNoteModal = ref(false);
const showEditNoteModal = ref(false);
const showViewNoteModal = ref(false);
const showDeleteNoteDialog = ref(false);
const editingNote = ref<Note | null>(null);
const viewingNote = ref<Note | null>(null);
const deletingNote = ref<Note | null>(null);

// Unlink confirm states
const unlinkType = ref<string>('');
const unlinkId = ref<string>('');
const unlinkName = ref<string>('');
const showUnlinkDialog = ref(false);
const unlinkMessage = computed(() => {
  if (unlinkType.value !== 'health') {
    return `Are you sure you want to unlink '${unlinkName.value}' from this application?`;
  } else {
    return `Are you sure you want to remove healthcheck '${unlinkName.value}'?`;
  }
});

const id = route.params.id as string;

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

// Link handlers
function openLinkModal(type: string) {
  linkStep.value = 'select';
  selectedEntity.value = null;
  initialName.value = '';
  switch (type) {
    case 'infra':
      showLinkInfraModal.value = true;
      break;
    case 'service':
      showLinkServiceModal.value = true;
      break;
    case 'domain':
      showLinkDomainModal.value = true;
      break;
    case 'person':
      showLinkPersonModal.value = true;
      break;
    case 'share':
      showLinkShareModal.value = true;
      break;
    case 'stack':
      showLinkStackModal.value = true;
      break;
    case 'health':
      showLinkHealthModal.value = true;
      break;
  }
}

function closeLinkModal(type: string) {
  switch (type) {
    case 'infra':
      showLinkInfraModal.value = false;
      break;
    case 'service':
      showLinkServiceModal.value = false;
      break;
    case 'domain':
      showLinkDomainModal.value = false;
      break;
    case 'person':
      showLinkPersonModal.value = false;
      break;
    case 'share':
      showLinkShareModal.value = false;
      break;
    case 'stack':
      showLinkStackModal.value = false;
      break;
    case 'health':
      showLinkHealthModal.value = false;
      break;
  }
}

function handleEntitySelect(entity: { id: string; name: string }) {
  selectedEntity.value = entity;
  linkStep.value = 'form';
}

function handleCreateRequest(searchTerm: string) {
  initialName.value = searchTerm;
  linkStep.value = 'create';
}

// Entity creation handlers
async function handleCreateInfra(data: CreateInfra) {
  try {
    const created = await infraApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create infra';
  }
}

async function handleCreateHealth(data: CreateHealthcheck) {
  try {
    await healthchecksApi.create(data);
    showLinkHealthModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to create healthcheck';
  }
}

async function handleCreateService(data: CreateService) {
  try {
    const created = await servicesApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create service';
  }
}

async function handleCreateDomain(data: CreateDomain) {
  try {
    const created = await domainsApi.create(data);
    selectedEntity.value = { id: created.id, name: created.fqdn };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create domain';
  }
}

async function handleCreatePerson(data: CreatePerson) {
  try {
    const created = await peopleApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create person';
  }
}

async function handleCreateShare(data: CreateNetworkShare) {
  try {
    const created = await sharesApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create share';
  }
}

async function handleCreateStack(data: CreateStack) {
  try {
    const created = await stacksApi.create(data);
    await applicationsApi.linkStack(id, created.id);
    showLinkStackModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create stack';
  }
}

async function handleStackSelect(entity: { id: string; name: string }) {
  try {
    await applicationsApi.linkStack(id, entity.id);
    showLinkStackModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link stack';
  }
}

// Link submission handlers
async function handleLinkInfra(data: LinkInfra) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkInfra(id, selectedEntity.value.id, data);
    showLinkInfraModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link infra';
  }
}

async function handleLinkService(data: LinkService) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkService(id, selectedEntity.value.id, data);
    showLinkServiceModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link service';
  }
}

async function handleLinkDomain(data: LinkDomain) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkDomain(id, selectedEntity.value.id, data);
    showLinkDomainModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link domain';
  }
}

async function handleLinkPerson(data: LinkPerson) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkPerson(id, selectedEntity.value.id, data);
    showLinkPersonModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link person';
  }
}

async function handleLinkShare(data: LinkNetworkShare) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkShare(id, selectedEntity.value.id, data);
    showLinkShareModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link share';
  }
}

// Edit handlers
function openEditInfra(infra: InfraRelation) {
  editingInfra.value = infra;
  showEditInfraModal.value = true;
}

// Edit handlers
async function openEditHealth(health: HealthcheckRelation) {
  try {
    // Fetch the full healthcheck data since HealthcheckRelation is missing fields
    const fullHealthcheck = await healthchecksApi.get(health.id);
    editingHealth.value = fullHealthcheck;
    showEditHealthModal.value = true;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load healthcheck';
  }
}

function openEditService(service: ServiceRelation) {
  editingService.value = service;
  showEditServiceModal.value = true;
}

function openEditDomain(domain: DomainRelation) {
  editingDomain.value = domain;
  showEditDomainModal.value = true;
}

function openEditPerson(person: PersonRelation) {
  editingPerson.value = person;
  showEditPersonModal.value = true;
}

function openEditShare(share: NetworkShareRelation) {
  editingShare.value = share;
  showEditShareModal.value = true;
}

async function handleEditInfra(data: LinkInfra) {
  if (!editingInfra.value) return;
  try {
    await applicationsApi.linkInfra(id, editingInfra.value.id, data);
    showEditInfraModal.value = false;
    editingInfra.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update infra link';
  }
}

async function handleEditHealth(data: UpdateHealthcheck) {
  if (!editingHealth.value) return;
  try {
    await healthchecksApi.update(editingHealth.value.id, data);
    showEditHealthModal.value = false;
    editingHealth.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update healthcheck';
  }
}

async function handleEditService(data: LinkService) {
  if (!editingService.value) return;
  try {
    await applicationsApi.linkService(id, editingService.value.id, data);
    showEditServiceModal.value = false;
    editingService.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update service link';
  }
}

async function handleEditDomain(data: LinkDomain) {
  if (!editingDomain.value) return;
  try {
    await applicationsApi.linkDomain(id, editingDomain.value.id, data);
    showEditDomainModal.value = false;
    editingDomain.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update domain link';
  }
}

async function handleEditPerson(data: LinkPerson) {
  if (!editingPerson.value) return;
  try {
    await applicationsApi.linkPerson(id, editingPerson.value.id, data);
    showEditPersonModal.value = false;
    editingPerson.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update person link';
  }
}

async function handleEditShare(data: LinkNetworkShare) {
  if (!editingShare.value) return;
  try {
    await applicationsApi.linkShare(id, editingShare.value.id, data);
    showEditShareModal.value = false;
    editingShare.value = null;
    loadData();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : 'Failed to update share link';
  }
}

// Note handlers
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

// Unlink handlers
function confirmUnlink(type: string, entityId: string, name: string) {
  unlinkType.value = type;
  unlinkId.value = entityId;
  unlinkName.value = name;
  showUnlinkDialog.value = true;
}

async function handleUnlink() {
  try {
    switch (unlinkType.value) {
      case 'infra':
        await applicationsApi.unlinkInfra(id, unlinkId.value);
        break;
      case 'service':
        await applicationsApi.unlinkService(id, unlinkId.value);
        break;
      case 'domain':
        await applicationsApi.unlinkDomain(id, unlinkId.value);
        break;
      case 'person':
        await applicationsApi.unlinkPerson(id, unlinkId.value);
        break;
      case 'share':
        await applicationsApi.unlinkShare(id, unlinkId.value);
        break;
      case 'stack':
        await applicationsApi.unlinkStack(id, unlinkId.value);
        break;
      case 'health':
        await healthchecksApi.delete(unlinkId.value);
        break;
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

    <div v-else-if="app">
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
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Services</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('service')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div
                v-if="app.services.length === 0"
                class="text-base-content/70"
              >
                No services linked
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th class="flex gap-2">
                        Name
                        <span class="badge badge-sm badge-neutral">env</span>
                      </th>
                      <th>Status</th>
                      <th>Notes</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="s in app.services" :key="s.id">
                      <td
                        class="cursor-pointer hover:text-primary"
                        @click="router.push(`/services/${s.id}`)"
                      >
                        <span class="flex gap-2">
                          {{ s.name }}
                          <EnvironmentBadge :environment="s.environment" />
                        </span>
                      </td>
                      <td><StatusBadge :status="s.status" /></td>
                      <td>{{ s.relation_notes || '-' }}</td>
                      <td class="flex justify-end">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click="openEditService(s)"
                        >
                          <Edit class="w-4 h-4" />
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click="confirmUnlink('service', s.id, s.name)"
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

          <!-- Domains Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Domains</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('domain')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div v-if="app.domains.length === 0" class="text-base-content/70">
                No domains linked
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th>Domain</th>
                      <th>Target</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="d in app.domains" :key="d.id">
                      <td
                        class="cursor-pointer hover:text-primary"
                        @click="router.push(`/domains/${d.id}`)"
                      >
                        {{ d.fqdn }}
                      </td>
                      <td
                        v-if="d.target_application_id"
                        class="cursor-pointer hover:text-primary hover:underline"
                        @click="
                          router.push(
                            `/applications/${d.target_application_id}`
                          )
                        "
                      >
                        {{ d.target_application_name }}
                        {{
                          d.target_application_name === app.name
                            ? '(this!)'
                            : ''
                        }}
                      </td>
                      <td
                        v-else-if="d.target_service_id"
                        class="cursor-pointer hover:text-primary hover:underline"
                        @click="router.push(`/services/${d.target_service_id}`)"
                      >
                        {{ d.target_service_name }}
                      </td>
                      <td class="flex justify-end">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click="openEditDomain(d)"
                        >
                          <Edit class="w-4 h-4" />
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click="confirmUnlink('domain', d.id, d.fqdn)"
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

          <!-- Infra Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Infrastructure</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('infra')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div v-if="app.infra.length === 0" class="text-base-content/70">
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
                    <tr v-for="i in app.infra" :key="i.id">
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
                      <td class="flex justify-end">
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

          <!-- Network Shares Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Storage</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('share')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div
                v-if="app.network_shares.length === 0"
                class="text-base-content/70"
              >
                No storage linked
              </div>
              <ul v-else class="space-y-2">
                <li v-for="s in app.network_shares" :key="s.id">
                  <div class="flex items-center justify-between">
                    <router-link
                      :to="`/shares/${s.id}`"
                      class="link link-hover"
                    >
                      {{ s.name }}
                    </router-link>
                    <div class="flex justify-end">
                      <button
                        class="btn btn-ghost btn-xs"
                        @click="openEditShare(s)"
                      >
                        <Edit class="w-4 h-4" />
                      </button>
                      <button
                        class="btn btn-ghost btn-xs text-error"
                        @click="confirmUnlink('share', s.id, s.name)"
                      >
                        <Link2Off class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                  <div class="text-xs text-base-content/70">{{ s.path }}</div>
                </li>
              </ul>
            </div>
          </div>

          <!-- Healthchecks Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Healthchecks</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('health')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div
                v-if="app.healthchecks.length === 0"
                class="text-base-content/70"
              >
                No healthchecks configured
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>URL</th>
                      <th>Expected</th>
                      <th>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="h in app.healthchecks"
                      :key="h.id"
                      class="cursor-pointer hover:bg-base-300"
                      @click="router.push(`/healthchecks/${h.id}`)"
                    >
                      <td>{{ h.name }}</td>
                      <td class="font-mono text-xs truncate max-w-50">
                        {{ h.protocol }}://{{ h.domain_fqdn }}{{ h.path }}
                      </td>
                      <td>{{ h.expected_status }}</td>
                      <td>
                        <StatusBadge
                          :status="h.is_enabled ? 'active' : 'inactive'"
                        />
                      </td>
                      <td class="flex justify-end">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click.stop="openEditHealth(h)"
                        >
                          <Edit class="w-4 h-4" />
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click.stop="confirmUnlink('health', h.id, h.name)"
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
          <!-- People Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">People</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('person')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div v-if="app.people.length === 0" class="text-base-content/70">
                No people linked
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="p in app.people"
                  :key="p.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/people/${p.id}`"
                      class="link link-hover"
                    >
                      {{ p.name }}
                    </router-link>
                    <span class="badge badge-outline badge-sm ml-2">
                      {{ p.contribution_type }}
                    </span>
                  </div>
                  <div class="flex justify-end">
                    <button
                      class="btn btn-ghost btn-xs"
                      @click="openEditPerson(p)"
                    >
                      <Edit class="w-4 h-4" />
                    </button>
                    <button
                      class="btn btn-ghost btn-xs text-error"
                      @click="confirmUnlink('person', p.id, p.name)"
                    >
                      <Link2Off class="w-4 h-4" />
                    </button>
                  </div>
                </li>
              </ul>
            </div>
          </div>

          <!-- Stack Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Tech Stack</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('stack')"
                >
                  <Plus class="w-4 h-4" /> Add
                </button>
              </div>
              <div v-if="app.stacks.length === 0" class="text-base-content/70">
                No technologies linked
              </div>
              <div v-else class="flex flex-wrap gap-2">
                <StackBadge
                  v-for="s in app.stacks"
                  @click="router.push(`/stack/${s.id}`)"
                  :key="s.id"
                  :name="s.name"
                  removable
                  clickable
                  @remove="confirmUnlink('stack', s.id, s.name)"
                />
              </div>
            </div>
          </div>

          <!-- Notes Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Notes</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="showCreateNoteModal = true"
                >
                  <Plus class="w-4 h-4" /> Add
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
    <Modal
      :title="linkStep === 'create' ? 'Create Infra' : 'Link Infra'"
      :open="showLinkInfraModal"
      @close="closeLinkModal('infra')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Infrastructure"
        :fetch-fn="infraApi.list"
        :exclude-ids="app?.infra.map((i) => i.id)"
        :allow-create="true"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('infra')"
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
        @cancel="closeLinkModal('infra')"
      />
    </Modal>

    <!-- Link Healthcheck Modal -->
    <Modal
      title="Create Healthcheck"
      :open="showLinkHealthModal"
      @close="closeLinkModal('health')"
    >
      <HealthcheckForm
        @submit="(data) => handleCreateHealth(data as CreateHealthcheck)"
        :initial-application-id="app?.id"
        :initial-name="app?.name"
        :initial-target-name="app?.name"
      />
    </Modal>

    <!-- Link Service Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Service' : 'Link Service'"
      :open="showLinkServiceModal"
      @close="closeLinkModal('service')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Services"
        :fetch-fn="servicesApi.list"
        :exclude-ids="app?.services.map((s) => s.id)"
        :allow-create="true"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('service')"
      />
      <ServiceForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateService(data as CreateService)"
        @cancel="linkStep = 'select'"
      />
      <LinkServiceForm
        v-else-if="selectedEntity"
        @submit="handleLinkService"
        @cancel="closeLinkModal('service')"
      />
    </Modal>

    <!-- Link Domain Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Domain' : 'Link Domain'"
      :open="showLinkDomainModal"
      @close="closeLinkModal('domain')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Domains"
        allow-create
        :fetch-fn="domainsApi.list"
        :exclude-ids="app?.domains.map((d) => d.id)"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('domain')"
      />
      <DomainForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateDomain(data as CreateDomain)"
        @cancel="linkStep = 'select'"
      />
      <LinkDomainForm
        v-else-if="selectedEntity"
        :domain-name="selectedEntity.name"
        @submit="handleLinkDomain"
        @cancel="closeLinkModal('domain')"
      />
    </Modal>

    <!-- Link Person Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Person' : 'Link Person'"
      :open="showLinkPersonModal"
      @close="closeLinkModal('person')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="People"
        allow-create
        :fetch-fn="peopleApi.list"
        :exclude-ids="app?.people.map((p) => p.id)"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('person')"
      />
      <PersonForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreatePerson(data as CreatePerson)"
        @cancel="linkStep = 'select'"
      />
      <LinkPersonForm
        v-else-if="selectedEntity"
        :person-name="selectedEntity.name"
        @submit="handleLinkPerson"
        @cancel="closeLinkModal('person')"
      />
    </Modal>

    <!-- Link Share Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Storage' : 'Link Storage'"
      :open="showLinkShareModal"
      @close="closeLinkModal('share')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Storage"
        allow-create
        :fetch-fn="sharesApi.list"
        :exclude-ids="app?.network_shares.map((s) => s.id)"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('share')"
      />
      <ShareForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateShare(data as CreateNetworkShare)"
        @cancel="linkStep = 'select'"
      />
      <LinkShareForm
        v-else-if="selectedEntity"
        :share-name="selectedEntity.name"
        @submit="handleLinkShare"
        @cancel="closeLinkModal('share')"
      />
    </Modal>

    <!-- Link Stack Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Stack' : 'Link Stack'"
      :open="showLinkStackModal"
      @close="closeLinkModal('stack')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Technology"
        allow-create
        :fetch-fn="stacksApi.list"
        :exclude-ids="app?.stacks.map((s) => s.id)"
        @select="handleStackSelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('stack')"
      />
      <StackForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateStack(data as CreateStack)"
        @cancel="linkStep = 'select'"
      />
    </Modal>

    <!-- Unlink Confirmation -->
    <ConfirmDialog
      :open="showUnlinkDialog"
      title="Unlink Entity"
      :message="unlinkMessage"
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

    <!-- Edit Health Modal -->
    <Modal
      title="Edit Healthcheck"
      :open="showEditHealthModal"
      @close="showEditHealthModal = false"
    >
      <HealthcheckForm
        v-if="editingHealth"
        :healthcheck="editingHealth"
        @submit="(data) => handleEditHealth(data as UpdateHealthcheck)"
        @cancel="showEditHealthModal = false"
      />
    </Modal>

    <!-- Edit Service Modal -->
    <Modal
      title="Edit Service Link"
      :open="showEditServiceModal"
      @close="showEditServiceModal = false"
    >
      <LinkServiceForm
        v-if="editingService"
        @submit="handleEditService"
        @cancel="showEditServiceModal = false"
      />
    </Modal>

    <!-- Edit Domain Modal -->
    <Modal
      title="Edit Domain Link"
      :open="showEditDomainModal"
      @close="showEditDomainModal = false"
    >
      <LinkDomainForm
        v-if="editingDomain"
        :domain-name="editingDomain.fqdn"
        :initial="editingDomain"
        @submit="handleEditDomain"
        @cancel="showEditDomainModal = false"
      />
    </Modal>

    <!-- Edit Person Modal -->
    <Modal
      title="Edit Person Link"
      :open="showEditPersonModal"
      @close="showEditPersonModal = false"
    >
      <LinkPersonForm
        v-if="editingPerson"
        :person-name="editingPerson.name"
        :initial="editingPerson"
        @submit="handleEditPerson"
        @cancel="showEditPersonModal = false"
      />
    </Modal>

    <!-- Edit Share Modal -->
    <Modal
      title="Edit Storage Link"
      :open="showEditShareModal"
      @close="showEditShareModal = false"
    >
      <LinkShareForm
        v-if="editingShare"
        :share-name="editingShare.name"
        :initial="editingShare"
        @submit="handleEditShare"
        @cancel="showEditShareModal = false"
      />
    </Modal>

    <!-- Create Note Modal -->
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

    <!-- Edit Note Modal -->
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

    <!-- View Note Modal -->
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

    <!-- Delete Note Confirmation -->
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
