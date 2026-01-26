<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  applicationsApi,
  hostsApi,
  domainsApi,
  peopleApi,
  clientsApi,
  sharesApi,
} from '@/api';
import type {
  ApplicationWithRelations,
  LinkHost,
  LinkDomain,
  LinkPerson,
  LinkClient,
  LinkNetworkShare,
  CreateHost,
  CreateDomain,
  CreatePerson,
  CreateClient,
  CreateNetworkShare,
  HostRelation,
  DomainRelation,
  PersonRelation,
  ClientRelation,
  NetworkShareRelation,
} from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import EntitySelector from '@/components/common/EntitySelector.vue';
import ApplicationForm from '@/components/forms/ApplicationForm.vue';
import HostForm from '@/components/forms/HostForm.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
import PersonForm from '@/components/forms/PersonForm.vue';
import ClientForm from '@/components/forms/ClientForm.vue';
import ShareForm from '@/components/forms/ShareForm.vue';
import LinkHostForm from '@/components/forms/LinkHostForm.vue';
import LinkDomainForm from '@/components/forms/LinkDomainForm.vue';
import LinkPersonForm from '@/components/forms/LinkPersonForm.vue';
import LinkClientForm from '@/components/forms/LinkClientForm.vue';
import LinkShareForm from '@/components/forms/LinkShareForm.vue';

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

const showLinkHostModal = ref(false);
const showLinkDomainModal = ref(false);
const showLinkPersonModal = ref(false);
const showLinkClientModal = ref(false);
const showLinkShareModal = ref(false);

// Edit modal states
const showEditHostModal = ref(false);
const showEditDomainModal = ref(false);
const showEditPersonModal = ref(false);
const showEditClientModal = ref(false);
const showEditShareModal = ref(false);
const editingHost = ref<HostRelation | null>(null);
const editingDomain = ref<DomainRelation | null>(null);
const editingPerson = ref<PersonRelation | null>(null);
const editingClient = ref<ClientRelation | null>(null);
const editingShare = ref<NetworkShareRelation | null>(null);

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
    case 'host':
      showLinkHostModal.value = true;
      break;
    case 'domain':
      showLinkDomainModal.value = true;
      break;
    case 'person':
      showLinkPersonModal.value = true;
      break;
    case 'client':
      showLinkClientModal.value = true;
      break;
    case 'share':
      showLinkShareModal.value = true;
      break;
  }
}

function closeLinkModal(type: string) {
  switch (type) {
    case 'host':
      showLinkHostModal.value = false;
      break;
    case 'domain':
      showLinkDomainModal.value = false;
      break;
    case 'person':
      showLinkPersonModal.value = false;
      break;
    case 'client':
      showLinkClientModal.value = false;
      break;
    case 'share':
      showLinkShareModal.value = false;
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
async function handleCreateHost(data: CreateHost) {
  try {
    const created = await hostsApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create host';
  }
}

async function handleCreateDomain(data: CreateDomain) {
  try {
    const created = await domainsApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
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

async function handleCreateClient(data: CreateClient) {
  try {
    const created = await clientsApi.create(data);
    selectedEntity.value = { id: created.id, name: created.name };
    linkStep.value = 'form';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create client';
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

// Link submission handlers
async function handleLinkHost(data: LinkHost) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkHost(id, selectedEntity.value.id, data);
    showLinkHostModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link host';
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

async function handleLinkClient(data: LinkClient) {
  if (!selectedEntity.value) return;
  try {
    await applicationsApi.linkClient(id, selectedEntity.value.id, data);
    showLinkClientModal.value = false;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to link client';
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
function openEditHost(host: HostRelation) {
  editingHost.value = host;
  showEditHostModal.value = true;
}

function openEditDomain(domain: DomainRelation) {
  editingDomain.value = domain;
  showEditDomainModal.value = true;
}

function openEditPerson(person: PersonRelation) {
  editingPerson.value = person;
  showEditPersonModal.value = true;
}

function openEditClient(client: ClientRelation) {
  editingClient.value = client;
  showEditClientModal.value = true;
}

function openEditShare(share: NetworkShareRelation) {
  editingShare.value = share;
  showEditShareModal.value = true;
}

async function handleEditHost(data: LinkHost) {
  if (!editingHost.value) return;
  try {
    await applicationsApi.linkHost(id, editingHost.value.id, data);
    showEditHostModal.value = false;
    editingHost.value = null;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update host link';
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
    error.value = e instanceof Error ? e.message : 'Failed to update domain link';
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
    error.value = e instanceof Error ? e.message : 'Failed to update person link';
  }
}

async function handleEditClient(data: LinkClient) {
  if (!editingClient.value) return;
  try {
    await applicationsApi.linkClient(id, editingClient.value.id, data);
    showEditClientModal.value = false;
    editingClient.value = null;
    loadData();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to update client link';
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
    error.value = e instanceof Error ? e.message : 'Failed to update share link';
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
      case 'host':
        await applicationsApi.unlinkHost(id, unlinkId.value);
        break;
      case 'domain':
        await applicationsApi.unlinkDomain(id, unlinkId.value);
        break;
      case 'person':
        await applicationsApi.unlinkPerson(id, unlinkId.value);
        break;
      case 'client':
        await applicationsApi.unlinkClient(id, unlinkId.value);
        break;
      case 'share':
        await applicationsApi.unlinkShare(id, unlinkId.value);
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
                <div>
                  <div class="text-sm text-base-content/70">Updated</div>
                  <div>{{ new Date(app.updated_at).toLocaleString() }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Hosts Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Hosts ({{ app.hosts.length }})</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('host')"
                >
                  + Add
                </button>
              </div>
              <div v-if="app.hosts.length === 0" class="text-base-content/70">
                No hosts linked
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Type</th>
                      <th>Hostname</th>
                      <th>Role</th>
                      <th>Status</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="h in app.hosts" :key="h.id">
                      <td
                        class="cursor-pointer hover:text-primary"
                        @click="router.push(`/hosts/${h.id}`)"
                      >
                        {{ h.name }}
                      </td>
                      <td>{{ h.host_type }}</td>
                      <td>{{ h.hostname || '-' }}</td>
                      <td>
                        <span class="badge badge-outline">{{ h.role }}</span>
                      </td>
                      <td><StatusBadge :status="h.status" /></td>
                      <td class="flex gap-1">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click="openEditHost(h)"
                        >
                          Edit
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click="confirmUnlink('host', h.id, h.name)"
                        >
                          Unlink
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
                <h2 class="card-title">Domains ({{ app.domains.length }})</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('domain')"
                >
                  + Add
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
                      <th>Record Type</th>
                      <th>Host</th>
                      <th>Primary</th>
                      <th>SSL Expires</th>
                      <th>Status</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="d in app.domains" :key="d.id">
                      <td
                        class="cursor-pointer hover:text-primary"
                        @click="router.push(`/domains/${d.id}`)"
                      >
                        {{ d.name }}
                      </td>
                      <td>{{ d.record_type }}</td>
                      <td>
                        <span
                          v-if="d.target_host_id"
                          class="cursor-pointer hover:text-primary underline"
                          @click="router.push(`/hosts/${d.target_host_id}`)"
                        >
                          {{ d.target_host_name }}
                        </span>
                        <span v-else-if="d.target">{{ d.target }}</span>
                        <span v-else>-</span>
                      </td>
                      <td>{{ d.is_primary ? 'Yes' : 'No' }}</td>
                      <td>{{ d.ssl_expires_at || '-' }}</td>
                      <td><StatusBadge :status="d.status" /></td>
                      <td class="flex gap-1">
                        <button
                          class="btn btn-ghost btn-xs"
                          @click="openEditDomain(d)"
                        >
                          Edit
                        </button>
                        <button
                          class="btn btn-ghost btn-xs text-error"
                          @click="confirmUnlink('domain', d.id, d.name)"
                        >
                          Unlink
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
                <h2 class="card-title">People ({{ app.people.length }})</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('person')"
                >
                  + Add
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
                  <div class="flex gap-1">
                    <button
                      class="btn btn-ghost btn-xs"
                      @click="openEditPerson(p)"
                    >
                      Edit
                    </button>
                    <button
                      class="btn btn-ghost btn-xs text-error"
                      @click="confirmUnlink('person', p.id, p.name)"
                    >
                      X
                    </button>
                  </div>
                </li>
              </ul>
            </div>
          </div>

          <!-- Clients Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">Clients ({{ app.clients.length }})</h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('client')"
                >
                  + Add
                </button>
              </div>
              <div v-if="app.clients.length === 0" class="text-base-content/70">
                No clients linked
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="c in app.clients"
                  :key="c.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/clients/${c.id}`"
                      class="link link-hover"
                    >
                      {{ c.name }}
                    </router-link>
                    <span class="badge badge-outline badge-sm ml-2">
                      {{ c.relationship_type }}
                    </span>
                  </div>
                  <div class="flex gap-1">
                    <button
                      class="btn btn-ghost btn-xs"
                      @click="openEditClient(c)"
                    >
                      Edit
                    </button>
                    <button
                      class="btn btn-ghost btn-xs text-error"
                      @click="confirmUnlink('client', c.id, c.name)"
                    >
                      X
                    </button>
                  </div>
                </li>
              </ul>
            </div>
          </div>

          <!-- Network Shares Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <div class="flex justify-between items-center">
                <h2 class="card-title">
                  Shares ({{ app.network_shares.length }})
                </h2>
                <button
                  class="btn btn-sm btn-ghost"
                  @click="openLinkModal('share')"
                >
                  + Add
                </button>
              </div>
              <div
                v-if="app.network_shares.length === 0"
                class="text-base-content/70"
              >
                No shares linked
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
                    <div class="flex gap-1">
                      <button
                        class="btn btn-ghost btn-xs"
                        @click="openEditShare(s)"
                      >
                        Edit
                      </button>
                      <button
                        class="btn btn-ghost btn-xs text-error"
                        @click="confirmUnlink('share', s.id, s.name)"
                      >
                        X
                      </button>
                    </div>
                  </div>
                  <div class="text-xs text-base-content/70">{{ s.path }}</div>
                </li>
              </ul>
            </div>
          </div>

          <!-- Notes Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Notes ({{ app.notes.length }})</h2>
              <div v-if="app.notes.length === 0" class="text-base-content/70">
                No notes
              </div>
              <ul v-else class="space-y-3">
                <li
                  v-for="n in app.notes"
                  :key="n.id"
                  class="border-b border-base-300 pb-2 last:border-0"
                >
                  <div class="font-medium">{{ n.title }}</div>
                  <div
                    v-if="n.content"
                    class="text-sm text-base-content/70 line-clamp-2"
                  >
                    {{ n.content }}
                  </div>
                  <a
                    v-if="n.url"
                    :href="n.url"
                    target="_blank"
                    class="link link-primary text-sm"
                    >View Link</a
                  >
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

    <!-- Link Host Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Host' : 'Link Host'"
      :open="showLinkHostModal"
      @close="closeLinkModal('host')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Hosts"
        :fetch-fn="hostsApi.list"
        :exclude-ids="app?.hosts.map((h) => h.id)"
        :allow-create="true"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('host')"
      />
      <HostForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateHost(data as CreateHost)"
        @cancel="linkStep = 'select'"
      />
      <LinkHostForm
        v-else-if="selectedEntity"
        :host-name="selectedEntity.name"
        @submit="handleLinkHost"
        @cancel="closeLinkModal('host')"
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

    <!-- Link Client Modal -->
    <Modal
      :title="linkStep === 'create' ? 'Create Client' : 'Link Client'"
      :open="showLinkClientModal"
      @close="closeLinkModal('client')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Clients"
        allow-create
        :fetch-fn="clientsApi.list"
        :exclude-ids="app?.clients.map((c) => c.id)"
        @select="handleEntitySelect"
        @create="handleCreateRequest"
        @cancel="closeLinkModal('client')"
      />
      <ClientForm
        v-else-if="linkStep === 'create'"
        :initial-name="initialName"
        @submit="(data) => handleCreateClient(data as CreateClient)"
        @cancel="linkStep = 'select'"
      />
      <LinkClientForm
        v-else-if="selectedEntity"
        :client-name="selectedEntity.name"
        @submit="handleLinkClient"
        @cancel="closeLinkModal('client')"
      />
    </Modal>

    <!-- Link Share Modal -->
    <Modal
      :title="
        linkStep === 'create' ? 'Create Network Share' : 'Link Network Share'
      "
      :open="showLinkShareModal"
      @close="closeLinkModal('share')"
    >
      <EntitySelector
        v-if="linkStep === 'select'"
        title="Network Shares"
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

    <!-- Unlink Confirmation -->
    <ConfirmDialog
      :open="showUnlinkDialog"
      title="Unlink Entity"
      :message="`Are you sure you want to unlink '${unlinkName}' from this application?`"
      confirm-label="Unlink"
      danger
      @confirm="handleUnlink"
      @cancel="showUnlinkDialog = false"
    />

    <!-- Edit Host Modal -->
    <Modal
      title="Edit Host Link"
      :open="showEditHostModal"
      @close="showEditHostModal = false"
    >
      <LinkHostForm
        v-if="editingHost"
        :host-name="editingHost.name"
        :initial="editingHost"
        @submit="handleEditHost"
        @cancel="showEditHostModal = false"
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
        :domain-name="editingDomain.name"
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

    <!-- Edit Client Modal -->
    <Modal
      title="Edit Client Link"
      :open="showEditClientModal"
      @close="showEditClientModal = false"
    >
      <LinkClientForm
        v-if="editingClient"
        :client-name="editingClient.name"
        :initial="editingClient"
        @submit="handleEditClient"
        @cancel="showEditClientModal = false"
      />
    </Modal>

    <!-- Edit Share Modal -->
    <Modal
      title="Edit Share Link"
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
  </div>
</template>
