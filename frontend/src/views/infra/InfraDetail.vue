<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { infraApi } from '@/api';
import type {
  InfraHealthcheckRelation,
  InfraWithRelations,
  LoadPoint,
} from '@/types';
import { useUptime } from '@/composables/useUptime';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import EnvironmentBadge from '@/components/common/EnvironmentBadge.vue';
import HealthPlot from '@/components/common/HealthPlot.vue';
import LoadChart from '@/components/common/LoadChart.vue';
import LoadBars from '@/components/common/LoadBars.vue';
import Modal from '@/components/common/Modal.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import InfraForm from '@/components/forms/InfraForm.vue';
import { infraTypes } from '@/values';
import { Bell } from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();

const infra = ref<InfraWithRelations | null>(null);
const loading = ref(true);
const error = ref('');
const showEditModal = ref(false);
const showDeleteDialog = ref(false);

const { monitors, getMonitorData } = useUptime();

type HealthStatus = 'up' | 'down' | 'pending' | 'unknown';

const healthcheckStatuses = computed(() => {
  if (!infra.value) return [];
  const _ = monitors.value;
  return infra.value.healthchecks
    .filter((hc) => hc.kuma_id != null)
    .map((hc) => {
      const data = getMonitorData(hc.kuma_id!);
      let status: HealthStatus = 'unknown';
      if (data && data.heartbeats.length > 0) {
        const last = data.heartbeats[data.heartbeats.length - 1];
        if (last.status === 1) status = 'up';
        else if (last.status === 0) status = 'down';
        else status = 'pending';
      }
      return { healthcheck: hc, status };
    });
});

function statusDotClass(status: HealthStatus): string {
  switch (status) {
    case 'up':
      return 'bg-success';
    case 'down':
      return 'bg-error';
    case 'pending':
      return 'bg-warning';
    default:
      return 'bg-base-content/30';
  }
}

function parentRoute(hc: InfraHealthcheckRelation): string {
  return hc.parent_type === 'application'
    ? `/applications/${hc.parent_id}`
    : `/services/${hc.parent_id}`;
}

function parentLabel(hc: InfraHealthcheckRelation): string {
  return hc.parent_type === 'application' ? 'App' : 'Service';
}

const id = route.params.id as string;

// ---- Zabbix load -------------------------------------------------------
const loadPoints = ref<LoadPoint[]>([]);
const loadHours = ref(24);
const hourOptions = [
  { label: '6h', value: 6 },
  { label: '24h', value: 24 },
  { label: '7d', value: 168 },
];
// Current = last sample of each metric (samples are interleaved per metric).
const currentLoad = computed(() => {
  const last = (key: 'cpu' | 'mem' | 'swap') => {
    for (let i = loadPoints.value.length - 1; i >= 0; i--) {
      const v = loadPoints.value[i][key];
      if (v != null) return v;
    }
    return null;
  };
  return { cpu: last('cpu'), mem: last('mem'), swap: last('swap') };
});
const hasLoad = computed(() => loadPoints.value.length > 0);
const loadReady = ref(false); // drives fade-in once data lands

function loadColor(v: number | null): string {
  if (v == null) return 'text-base-content/40';
  if (v < 70) return 'text-success';
  if (v < 90) return 'text-warning';
  return 'text-error';
}

async function fetchLoad() {
  try {
    const points = await infraApi.loadHistory(id, loadHours.value);
    loadReady.value = false;
    loadPoints.value = points;
    // Wait for the card to mount at opacity-0, then fade in.
    await nextTick();
    requestAnimationFrame(() => (loadReady.value = true));
  } catch {
    loadPoints.value = [];
  }
}

async function loadData() {
  loading.value = true;
  error.value = '';
  try {
    infra.value = await infraApi.get(id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load infra';
  } finally {
    loading.value = false;
  }
}

async function handleUpdate(formData: unknown) {
  try {
    await infraApi.update(
      id,
      formData as Parameters<typeof infraApi.update>[1]
    );
    showEditModal.value = false;
    toast.success('Infrastructure updated');
    loadData();
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to update infra');
  }
}

const syncLoading = ref(false);
async function syncIps() {
  syncLoading.value = true;
  try {
    const promise = infraApi.syncOne(id);
    toast.promise(promise, {
      loading: 'Resolving IPs…',
      success: 'IPs synced',
      error: (e: unknown) => (e instanceof Error ? e.message : 'Sync failed'),
    });
    infra.value = await promise;
  } catch {
    // toast.promise already surfaced the error
  } finally {
    syncLoading.value = false;
  }
}

async function handleDelete() {
  try {
    await infraApi.delete(id);
    toast.success('Infrastructure deleted');
    router.push('/infra');
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to delete infra');
  }
}

onMounted(loadData);
onMounted(fetchLoad);

function setHours(h: number) {
  loadHours.value = h;
  fetchLoad();
}

// Keyboard shortcuts
function handleGlobalKeydown(e: KeyboardEvent) {
  if (
    e.target instanceof HTMLInputElement ||
    e.target instanceof HTMLTextAreaElement ||
    e.target instanceof HTMLSelectElement
  )
    return;
  if (e.key === 'e' && !showEditModal.value) {
    e.preventDefault();
    showEditModal.value = true;
  }
}

onMounted(() => document.addEventListener('keydown', handleGlobalKeydown));
onUnmounted(() => document.removeEventListener('keydown', handleGlobalKeydown));
</script>

<template>
  <div class="p-6">
    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="infra">
      <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6"
      >
        <div>
          <div class="breadcrumbs text-sm mb-2">
            <ul>
              <li>
                <router-link to="/infra">Infrastructure</router-link>
              </li>
              <li>{{ infra.name }}</li>
            </ul>
          </div>
          <h1 class="text-2xl font-bold flex items-center gap-3">
            {{ infra.name }}
            <span class="badge badge-outline">{{
              infraTypes[infra.type as keyof typeof infraTypes] || infra.type
            }}</span>
          </h1>
        </div>
        <div class="flex gap-2">
          <button class="btn btn-sm" :disabled="syncLoading" @click="syncIps">
            <span
              v-if="syncLoading"
              class="loading loading-spinner loading-xs"
            />
            Sync IPs
          </button>
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
                  <div>{{ infra.description || '-' }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Type</div>
                  <div>
                    {{
                      infraTypes[infra.type as keyof typeof infraTypes] ||
                      infra.type
                    }}
                  </div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Created</div>
                  <div>{{ new Date(infra.created_at).toLocaleString() }}</div>
                </div>
                <div>
                  <div class="text-sm text-base-content/70">Updated</div>
                  <div>{{ new Date(infra.updated_at).toLocaleString() }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- IP Addresses Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">IP Addresses ({{ infra.ips.length }})</h2>
              <div v-if="infra.ips.length === 0" class="text-base-content/70">
                No IPs — attach a domain or add manual IPs via Edit.
              </div>
              <div v-else class="overflow-x-auto">
                <table class="table table-sm">
                  <thead>
                    <tr>
                      <th>IP</th>
                      <th>Source</th>
                      <th>Synced</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="ip in infra.ips" :key="ip.ip">
                      <td class="font-mono break-all">{{ ip.ip }}</td>
                      <td>
                        <span
                          class="badge badge-sm"
                          :class="
                            ip.source === 'domain'
                              ? 'badge-info'
                              : 'badge-ghost'
                          "
                          >{{ ip.source }}</span
                        >
                      </td>
                      <td class="text-xs text-base-content/60">
                        {{ new Date(ip.last_synced_at).toLocaleString() }}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          <!-- Health Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Health</h2>
              <div
                v-if="infra.healthchecks.length === 0"
                class="text-base-content/70"
              >
                No healthchecks from linked applications or services
              </div>
              <div v-else class="grid grid-cols-1 gap-y-1">
                <div
                  v-for="item in healthcheckStatuses"
                  :key="item.healthcheck.id"
                  class="grid grid-cols-1 md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)] gap-4 py-2 border-b border-base-300 last:border-b-0"
                >
                  <div
                    class="flex md:flex-col justify-between md:justify-center gap-1 min-w-0"
                  >
                    <div class="flex items-center gap-2 min-w-0">
                      <span
                        class="w-2.5 h-2.5 rounded-full shrink-0"
                        :class="statusDotClass(item.status)"
                      />
                      <router-link
                        :to="`/healthchecks/${item.healthcheck.id}`"
                        class="link link-hover truncate"
                      >
                        {{ item.healthcheck.name }}
                      </router-link>
                      <span
                        v-if="item.healthcheck.notifications"
                        title="Notifications enabled"
                        class="inline-flex shrink-0"
                      >
                        <Bell class="w-3.5 h-3.5 text-primary" />
                      </span>
                    </div>
                    <div
                      class="flex max-md:flex-row-reverse items-order items-center gap-2 pl-4"
                    >
                      <span class="badge badge-xs badge-outline">{{
                        parentLabel(item.healthcheck)
                      }}</span>
                      <router-link
                        :to="parentRoute(item.healthcheck)"
                        class="link link-hover text-xs truncate"
                      >
                        {{ item.healthcheck.parent_name }}
                      </router-link>
                    </div>
                  </div>
                  <div class="h-4 w-full shrink-0">
                    <HealthPlot
                      :kuma-id="item.healthcheck.kuma_id!"
                      :tick-width="6"
                    />
                  </div>
                </div>
                <div
                  v-if="
                    infra.healthchecks.length > 0 &&
                    healthcheckStatuses.length === 0
                  "
                  class="text-base-content/70 text-sm"
                >
                  Linked healthchecks are not synced to Kuma yet
                </div>
              </div>
            </div>
          </div>

          <!-- Load Card (Zabbix) -->
          <div
            v-if="hasLoad"
            class="card bg-base-200 transition-opacity duration-500"
            :class="loadReady ? 'opacity-100' : 'opacity-0'"
          >
            <div class="card-body">
              <div class="flex items-center justify-between">
                <h2 class="card-title">Load</h2>
                <div class="join">
                  <button
                    v-for="opt in hourOptions"
                    :key="opt.value"
                    class="btn btn-xs join-item"
                    :class="loadHours === opt.value ? 'btn-active' : ''"
                    @click="setHours(opt.value)"
                  >
                    {{ opt.label }}
                  </button>
                </div>
              </div>
              <div class="flex flex-col gap-6">
                <div class="flex gap-3">
                  current:
                  <LoadBars
                    :load="currentLoad"
                    orientation="horizontal"
                    :numbers="true"
                  />
                </div>
                <LoadChart :points="loadPoints" />
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-6">
          <!-- Applications Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Applications</h2>
              <div
                v-if="infra.applications.length === 0"
                class="text-base-content/70"
              >
                No applications using this infrastructure
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="a in infra.applications"
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

          <!-- Services Card -->
          <div class="card bg-base-200">
            <div class="card-body">
              <h2 class="card-title">Services</h2>
              <div
                v-if="infra.services.length === 0"
                class="text-base-content/70"
              >
                No services using this infrastructure
              </div>
              <ul v-else class="space-y-2">
                <li
                  v-for="s in infra.services"
                  :key="s.id"
                  class="flex items-center justify-between"
                >
                  <div>
                    <router-link
                      :to="`/services/${s.id}`"
                      class="link link-hover"
                    >
                      {{ s.name }}
                    </router-link>
                    <EnvironmentBadge
                      :environment="s.environment"
                      class="ml-2"
                    />
                  </div>
                  <StatusBadge :status="s.status" />
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Infra Modal -->
    <Modal
      title="Edit Infra"
      :open="showEditModal"
      @close="showEditModal = false"
    >
      <InfraForm
        v-if="infra"
        :infra="infra"
        @submit="handleUpdate"
        @cancel="showEditModal = false"
      />
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Infrastructure"
      message="Are you sure you want to delete this infrastructure? This action cannot be undone."
      confirm-label="Delete"
      danger
      @confirm="handleDelete"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>
