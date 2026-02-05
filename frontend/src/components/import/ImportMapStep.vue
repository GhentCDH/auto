<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import type {
  KumaMonitorImport,
  ImportMapping,
  Domain,
  CreateApplication,
  CreateService,
} from '@/types';
import EntitySelector from '../common/EntitySelector.vue';
import Modal from '../common/Modal.vue';
import ApplicationForm from '../forms/ApplicationForm.vue';
import ServiceForm from '../forms/ServiceForm.vue';
import { applicationsApi, servicesApi, domainsApi } from '@/api';
import { X, Check } from 'lucide-vue-next';

// Side panel width in rem (w-72 = 18rem, plus ml-4 + pl-4 = 2rem)
const SIDE_PANEL_WIDTH_REM = 35;

const props = defineProps<{
  monitor: KumaMonitorImport;
  index: number;
  total: number;
  existingMapping?: ImportMapping;
}>();

const emit = defineEmits<{
  mapped: [mapping: ImportMapping];
  skip: [];
  back: [];
  panelChange: [isOpen: boolean, widthRem: number];
}>();

// Domain state
const domainLoading = ref(true);
const matchedDomain = ref<Domain | null>(null);
const createNewDomain = ref(false);

const panelStyle = `width: ${SIDE_PANEL_WIDTH_REM}rem`;

// Domain target state (for new domains)
const domainTargetPanelOpen = ref(false);
const domainTargetType = ref<'application' | 'service'>('application');
const selectedDomainTargetId = ref<string | null>(null);
const selectedDomainTargetName = ref<string | null>(null);

// Healthcheck target state
const healthcheckTargetPanelOpen = ref(false);
const healthcheckTargetType = ref<'application' | 'service'>('application');
const selectedHealthcheckTargetId = ref<string | null>(null);
const selectedHealthcheckTargetName = ref<string | null>(null);

// Create entity modals
const showCreateApplication = ref(false);
const showCreateService = ref(false);
const createTargetContext = ref<'domain' | 'healthcheck'>('healthcheck');
const pendingCreateName = ref('');

// Skip state
const skipMonitor = ref(false);

// Check if any panel is open
const hasPanelOpen = computed(
  () => domainTargetPanelOpen.value || healthcheckTargetPanelOpen.value
);

// Emit panel state changes
watch(hasPanelOpen, (isOpen) => {
  emit('panelChange', isOpen, SIDE_PANEL_WIDTH_REM);
});

const isValid = computed(() => {
  if (skipMonitor.value) return true;
  const hasDomain = matchedDomain.value !== null || createNewDomain.value;
  const hasHealthcheckTarget = selectedHealthcheckTargetId.value !== null;
  // If creating a new domain, we also need a domain target
  const hasDomainTarget =
    !createNewDomain.value || selectedDomainTargetId.value !== null;
  return hasDomain && hasHealthcheckTarget && hasDomainTarget;
});

async function lookupDomain() {
  domainLoading.value = true;
  matchedDomain.value = null;

  try {
    const result = await domainsApi.list({ search: props.monitor.hostname });
    const exactMatch = result.data.find(
      (d) => d.fqdn === props.monitor.hostname
    );
    if (exactMatch) {
      matchedDomain.value = exactMatch as Domain;
    }
  } catch {
    // Domain not found - that's okay, user can create it
  } finally {
    domainLoading.value = false;
  }
}

function handleDomainTargetSelect(entity: { id: string; name: string }) {
  selectedDomainTargetId.value = entity.id;
  selectedDomainTargetName.value = entity.name;
  domainTargetPanelOpen.value = false;
}

function handleHealthcheckTargetSelect(entity: { id: string; name: string }) {
  selectedHealthcheckTargetId.value = entity.id;
  selectedHealthcheckTargetName.value = entity.name;
  healthcheckTargetPanelOpen.value = false;
}

function handleNext() {
  if (skipMonitor.value) {
    emit('mapped', {
      monitor: props.monitor,
      domain_id: null,
      domain_fqdn: props.monitor.hostname,
      application_id: null,
      service_id: null,
      target_name: '',
      domain_application_id: null,
      domain_service_id: null,
      domain_target_name: '',
      skip: true,
    });
    return;
  }

  const mapping: ImportMapping = {
    monitor: props.monitor,
    domain_id: matchedDomain.value?.id || null,
    domain_fqdn: props.monitor.hostname,
    // Healthcheck target
    application_id:
      healthcheckTargetType.value === 'application'
        ? selectedHealthcheckTargetId.value
        : null,
    service_id:
      healthcheckTargetType.value === 'service'
        ? selectedHealthcheckTargetId.value
        : null,
    target_name: selectedHealthcheckTargetName.value || '',
    // Domain target
    domain_application_id:
      createNewDomain.value && domainTargetType.value === 'application'
        ? selectedDomainTargetId.value
        : null,
    domain_service_id:
      createNewDomain.value && domainTargetType.value === 'service'
        ? selectedDomainTargetId.value
        : null,
    domain_target_name: createNewDomain.value
      ? selectedDomainTargetName.value || ''
      : '',
    skip: false,
  };

  emit('mapped', mapping);
}

function handleSkipAll() {
  emit('skip');
}

function openDomainTargetPanel() {
  healthcheckTargetPanelOpen.value = false;
  domainTargetPanelOpen.value = true;
}

function openHealthcheckTargetPanel() {
  domainTargetPanelOpen.value = false;
  healthcheckTargetPanelOpen.value = true;
}

function handleCreateFromDomainPanel(searchTerm: string) {
  createTargetContext.value = 'domain';
  pendingCreateName.value = searchTerm;
  domainTargetPanelOpen.value = false;
  if (domainTargetType.value === 'application') {
    showCreateApplication.value = true;
  } else {
    showCreateService.value = true;
  }
}

function handleCreateFromHealthcheckPanel(searchTerm: string) {
  createTargetContext.value = 'healthcheck';
  pendingCreateName.value = searchTerm;
  healthcheckTargetPanelOpen.value = false;
  if (healthcheckTargetType.value === 'application') {
    showCreateApplication.value = true;
  } else {
    showCreateService.value = true;
  }
}

async function handleCreateApplication(data: CreateApplication) {
  const created = await applicationsApi.create(data);

  if (createTargetContext.value === 'domain') {
    selectedDomainTargetId.value = created.id;
    selectedDomainTargetName.value = created.name;
    domainTargetType.value = 'application';
  } else {
    selectedHealthcheckTargetId.value = created.id;
    selectedHealthcheckTargetName.value = created.name;
    healthcheckTargetType.value = 'application';
  }

  showCreateApplication.value = false;
  pendingCreateName.value = '';
}

async function handleCreateService(data: CreateService) {
  const created = await servicesApi.create(data);

  if (createTargetContext.value === 'domain') {
    selectedDomainTargetId.value = created.id;
    selectedDomainTargetName.value = created.name;
    domainTargetType.value = 'service';
  } else {
    selectedHealthcheckTargetId.value = created.id;
    selectedHealthcheckTargetName.value = created.name;
    healthcheckTargetType.value = 'service';
  }

  showCreateService.value = false;
  pendingCreateName.value = '';
}

// Restore existing mapping if provided
watch(
  () => props.existingMapping,
  (mapping) => {
    if (mapping && !mapping.skip) {
      // Restore healthcheck target
      if (mapping.application_id) {
        healthcheckTargetType.value = 'application';
        selectedHealthcheckTargetId.value = mapping.application_id;
      } else if (mapping.service_id) {
        healthcheckTargetType.value = 'service';
        selectedHealthcheckTargetId.value = mapping.service_id;
      }
      selectedHealthcheckTargetName.value = mapping.target_name;

      // Restore domain target
      if (mapping.domain_application_id) {
        domainTargetType.value = 'application';
        selectedDomainTargetId.value = mapping.domain_application_id;
      } else if (mapping.domain_service_id) {
        domainTargetType.value = 'service';
        selectedDomainTargetId.value = mapping.domain_service_id;
      }
      selectedDomainTargetName.value = mapping.domain_target_name;

      createNewDomain.value = mapping.domain_id === null;
    }
    skipMonitor.value = mapping?.skip || false;
  },
  { immediate: true }
);

onMounted(() => {
  lookupDomain();
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (domainTargetPanelOpen.value) {
      domainTargetPanelOpen.value = false;
    } else if (healthcheckTargetPanelOpen.value) {
      healthcheckTargetPanelOpen.value = false;
    }
  }
}
</script>

<template>
  <div class="flex min-w-0">
    <!-- Main content - fixed width, doesn't resize when panel opens -->
    <div class="w-0 flex-1 flex flex-col gap-4">
      <!-- Progress indicator -->
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5">
          <span class="text-sm font-medium">{{ index + 1 }}</span>
          <span class="text-sm text-base-content/50">/</span>
          <span class="text-sm text-base-content/50">{{ total }}</span>
        </div>
        <progress
          class="progress progress-primary flex-1 h-2"
          :value="index + 1"
          :max="total"
        />
        <span class="text-xs text-base-content/50">
          {{ Math.round(((index + 1) / total) * 100) }}%
        </span>
      </div>

      <!-- Monitor info card -->
      <div class="card bg-base-200 p-4">
        <h3 class="font-medium text-lg">{{ monitor.name }}</h3>
        <p class="text-sm text-base-content/70 font-mono break-all">
          {{ monitor.url }}
        </p>
        <div class="flex gap-4 mt-2 text-sm">
          <span class="badge badge-outline">{{ monitor.method }}</span>
          <span class="badge badge-outline"
            >Status: {{ monitor.expected_status }}</span
          >
          <span v-if="monitor.keyword" class="badge badge-outline">
            Body match: "{{ monitor.keyword }}"
          </span>
        </div>
      </div>

      <!-- Skip option -->
      <label
        class="label cursor-pointer justify-start gap-3 bg-base-200 rounded-box p-3"
      >
        <input
          type="checkbox"
          v-model="skipMonitor"
          class="checkbox checkbox-sm"
        />
        <span>Skip this monitor</span>
      </label>

      <template v-if="!skipMonitor">
        <!-- Domain section -->
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Domain</legend>
          <div
            class="bg-base-200 rounded-box p-3 transition-shadow"
            :class="{ 'ring-2 ring-primary': domainTargetPanelOpen }"
          >
            <div v-if="domainLoading" class="flex items-center gap-2">
              <div class="skeleton h-5 w-16 rounded"></div>
              <div class="skeleton h-4 w-40 rounded"></div>
            </div>
            <div v-else-if="matchedDomain" class="flex items-center gap-2">
              <span class="badge badge-success">Found</span>
              <span class="font-mono">{{ matchedDomain.fqdn }}</span>
            </div>
            <div v-else class="space-y-3">
              <div class="flex items-center gap-2">
                <span class="badge badge-warning">Not found</span>
                <span class="font-mono">{{ monitor.hostname }}</span>
              </div>
              <label class="label cursor-pointer justify-start gap-3">
                <input
                  type="checkbox"
                  v-model="createNewDomain"
                  class="checkbox checkbox-sm checkbox-primary"
                />
                <span class="text-sm">
                  Create domain
                  <span class="font-mono">{{ monitor.hostname }}</span>
                </span>
              </label>

              <!-- Domain target selection (when creating new domain) -->
              <div v-if="createNewDomain" class="pl-7 space-y-2">
                <p class="text-sm text-base-content/70">
                  Select target for new domain:
                </p>
                <div
                  v-if="selectedDomainTargetId"
                  class="flex items-center justify-between bg-success/10 border border-success/30 rounded-box p-2"
                >
                  <div class="flex items-center gap-2">
                    <Check class="w-4 h-4 text-success" />
                    <span class="badge badge-primary badge-sm">
                      {{ domainTargetType === 'application' ? 'App' : 'Svc' }}
                    </span>
                    <span class="font-medium text-sm">{{
                      selectedDomainTargetName
                    }}</span>
                  </div>
                  <button
                    type="button"
                    class="btn btn-ghost btn-xs"
                    @click="openDomainTargetPanel()"
                  >
                    Change
                  </button>
                </div>
                <div v-else>
                  <button
                    type="button"
                    class="btn btn-outline btn-sm"
                    @click="openDomainTargetPanel()"
                  >
                    Select Target
                  </button>
                </div>
              </div>
            </div>
          </div>
        </fieldset>

        <!-- Healthcheck Target section -->
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Healthcheck Target *</legend>
          <div
            class="bg-base-200 rounded-box p-3 transition-shadow"
            :class="{ 'ring-2 ring-primary': healthcheckTargetPanelOpen }"
          >
            <div
              v-if="selectedHealthcheckTargetId"
              class="flex items-center justify-between bg-success/10 border border-success/30 rounded-box p-2"
            >
              <div class="flex items-center gap-2">
                <Check class="w-4 h-4 text-success" />
                <span class="badge badge-primary badge-sm">
                  {{ healthcheckTargetType === 'application' ? 'App' : 'Svc' }}
                </span>
                <span class="font-medium text-sm">{{
                  selectedHealthcheckTargetName
                }}</span>
              </div>
              <button
                type="button"
                class="btn btn-ghost btn-xs"
                @click="openHealthcheckTargetPanel()"
              >
                Change
              </button>
            </div>
            <div v-else>
              <button
                type="button"
                class="btn btn-outline btn-sm"
                @click="openHealthcheckTargetPanel()"
              >
                Select Target
              </button>
            </div>
          </div>
        </fieldset>
      </template>

      <!-- Navigation -->
      <div class="flex justify-between gap-2 pt-4">
        <button type="button" class="btn btn-ghost" @click="emit('back')">
          Back
        </button>
        <div class="flex gap-2">
          <button
            type="button"
            class="btn btn-secondary"
            @click="handleSkipAll"
          >
            Skip Rest
          </button>
          <button
            type="button"
            class="btn btn-primary"
            :disabled="!isValid"
            @click="handleNext"
          >
            {{ index + 1 === total ? 'Review' : 'Next' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Side Panel for Domain Target -->
    <Transition name="slide">
      <div
        v-if="domainTargetPanelOpen"
        class="shrink-0 border-l border-base-300 ml-4 pl-4 flex flex-col"
        :style="panelStyle"
      >
        <div class="flex items-center justify-between mb-3">
          <h4 class="font-semibold text-base">Select Domain Target</h4>
          <button
            type="button"
            class="btn btn-ghost btn-xs btn-circle"
            @click="domainTargetPanelOpen = false"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="join w-full mb-4">
          <button
            type="button"
            class="btn btn-sm join-item flex-1"
            :class="
              domainTargetType === 'application'
                ? 'btn-primary'
                : 'btn-ghost bg-base-200'
            "
            @click="domainTargetType = 'application'"
          >
            Application
          </button>
          <button
            type="button"
            class="btn btn-sm join-item flex-1"
            :class="
              domainTargetType === 'service'
                ? 'btn-primary'
                : 'btn-ghost bg-base-200'
            "
            @click="domainTargetType = 'service'"
          >
            Service
          </button>
        </div>

        <EntitySelector
          v-if="domainTargetType === 'application'"
          title="Applications"
          :fetch-fn="applicationsApi.list"
          allow-create
          @select="handleDomainTargetSelect"
          @cancel="domainTargetPanelOpen = false"
          @create="handleCreateFromDomainPanel"
        />
        <EntitySelector
          v-else
          title="Services"
          :fetch-fn="servicesApi.list"
          allow-create
          @select="handleDomainTargetSelect"
          @cancel="domainTargetPanelOpen = false"
          @create="handleCreateFromDomainPanel"
        />
      </div>
    </Transition>

    <!-- Side Panel for Healthcheck Target -->
    <Transition name="slide">
      <div
        v-if="healthcheckTargetPanelOpen"
        class="shrink-0 border-l border-base-300 ml-4 pl-4 flex flex-col"
        :style="panelStyle"
      >
        <div class="flex items-center justify-between mb-3">
          <h4 class="font-semibold text-base">Select Healthcheck Target</h4>
          <button
            type="button"
            class="btn btn-ghost btn-xs btn-circle"
            @click="healthcheckTargetPanelOpen = false"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="join w-full mb-4">
          <button
            type="button"
            class="btn btn-sm join-item flex-1"
            :class="
              healthcheckTargetType === 'application'
                ? 'btn-primary'
                : 'btn-ghost bg-base-200'
            "
            @click="healthcheckTargetType = 'application'"
          >
            Application
          </button>
          <button
            type="button"
            class="btn btn-sm join-item flex-1"
            :class="
              healthcheckTargetType === 'service'
                ? 'btn-primary'
                : 'btn-ghost bg-base-200'
            "
            @click="healthcheckTargetType = 'service'"
          >
            Service
          </button>
        </div>

        <EntitySelector
          v-if="healthcheckTargetType === 'application'"
          title="Applications"
          :fetch-fn="applicationsApi.list"
          allow-create
          @select="handleHealthcheckTargetSelect"
          @cancel="healthcheckTargetPanelOpen = false"
          @create="handleCreateFromHealthcheckPanel"
        />
        <EntitySelector
          v-else
          title="Services"
          :fetch-fn="servicesApi.list"
          allow-create
          @select="handleHealthcheckTargetSelect"
          @cancel="healthcheckTargetPanelOpen = false"
          @create="handleCreateFromHealthcheckPanel"
        />
      </div>
    </Transition>
  </div>

  <!-- Create Application Modal -->
  <Modal
    title="Create Application"
    :open="showCreateApplication"
    @close="showCreateApplication = false"
  >
    <ApplicationForm
      :initial-name="pendingCreateName"
      @submit="(data) => handleCreateApplication(data as CreateApplication)"
      @cancel="showCreateApplication = false"
    />
  </Modal>

  <!-- Create Service Modal -->
  <Modal
    title="Create Service"
    :open="showCreateService"
    @close="showCreateService = false"
  >
    <ServiceForm
      :initial-name="pendingCreateName"
      @submit="(data) => handleCreateService(data as CreateService)"
      @cancel="showCreateService = false"
    />
  </Modal>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: all 0.1s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
