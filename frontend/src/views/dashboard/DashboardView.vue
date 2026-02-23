<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { dashboardApi, healthchecksApi } from '@/api';
import type { DashboardStats, Healthcheck } from '@/types';
import type { ComponentPublicInstance } from 'vue';
import { useUptime } from '@/composables/useUptime';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import HealthPlot from '@/components/common/HealthPlot.vue';

const stats = ref<DashboardStats | null>(null);
const healthchecks = ref<Healthcheck[]>([]);
const loading = ref(true);
const error = ref('');

const { monitors, getMonitorData } = useUptime();

function toTitleCase(input: string): string {
  return input
    .split(' ')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
}

onMounted(async () => {
  try {
    const [dashStats, hcResponse] = await Promise.all([
      dashboardApi.getStats(),
      healthchecksApi.list({ per_page: 200, is_enabled: true }),
    ]);
    stats.value = dashStats;
    healthchecks.value = hcResponse.data;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load dashboard';
  } finally {
    loading.value = false;
  }
});

type HealthStatus = 'up' | 'down' | 'pending' | 'unknown';

const healthcheckStatuses = computed(() => {
  // Access monitors to trigger reactivity
  const _ = monitors.value;
  return healthchecks.value
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

const downHealthchecks = computed(() =>
  healthcheckStatuses.value.filter((h) => h.status === 'down')
);

const pendingHealthchecks = computed(() =>
  healthcheckStatuses.value.filter((h) => h.status === 'pending')
);

function entityRoute(item: { entity_type: string; id: string }): string {
  const routes: Record<string, string> = {
    application: 'applications',
    service: 'services',
    infra: 'infra',
    domain: 'domains',
    person: 'people',
    network_share: 'shares',
    healthcheck: 'healthchecks',
  };
  const base = routes[item.entity_type] ?? item.entity_type;
  return `/${base}/${item.id}`;
}

function entityLabel(type: string): string {
  const labels: Record<string, string> = {
    application: 'App',
    service: 'Service',
    infra: 'Infra',
    domain: 'Domain',
    person: 'Person',
    network_share: 'Share',
    healthcheck: 'Check',
  };
  return labels[type] ?? type;
}

function entityBadgeClass(type: string): string {
  const classes: Record<string, string> = {
    application: 'badge-primary',
    service: 'badge-secondary',
    infra: 'badge-accent',
    domain: 'badge-info',
    person: 'badge-warning',
    network_share: 'badge-error',
    healthcheck: 'badge-success',
  };
  return classes[type] ?? 'badge-neutral';
}

function formatRelativeTime(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;
  const mins = Math.floor(diffMs / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function formatDate(date: string | null): string {
  if (!date) return 'N/A';
  return new Date(date).toLocaleDateString();
}

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

function healthcheckTarget(hc: Healthcheck): string {
  if (hc.application_id) return `/applications/${hc.application_id}`;
  if (hc.service_id) return `/services/${hc.service_id}`;
  return `/healthchecks/${hc.id}`;
}

const linkRef = ref<ComponentPublicInstance | null>(null);
const textRef = ref<HTMLSpanElement | null>(null);
const isOverflowing = ref(false);

onMounted(() => {
  const container = (linkRef.value?.$el as HTMLElement | undefined) ?? null;
  const text = textRef.value;
  if (container && text) {
    const overflow = text.scrollWidth - container.clientWidth;
    if (overflow > 0) {
      isOverflowing.value = true;
      text.style.setProperty('--scroll-dist', `-${overflow}px`);
    }
  }
});
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>

    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="stats">
      <!-- Down/Degraded Alert -->
      <div v-if="downHealthchecks.length > 0" class="alert alert-error mb-6">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6 shrink-0 stroke-current"
          fill="none"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
        <div>
          <span class="font-semibold"
            >{{ downHealthchecks.length }} service{{
              downHealthchecks.length > 1 ? 's' : ''
            }}
            down:</span
          >
          <div class="flex flex-wrap gap-1 mt-1">
            <router-link
              v-for="item in downHealthchecks"
              :key="item.healthcheck.id"
              :to="healthcheckTarget(item.healthcheck)"
              class="badge badge-warning gap-1"
            >
              {{ item.healthcheck.name }}
            </router-link>
          </div>
        </div>
      </div>

      <div
        v-if="pendingHealthchecks.length > 0"
        class="alert alert-warning mb-6"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6 shrink-0 stroke-current"
          fill="none"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
        <div>
          <span class="font-semibold"
            >{{ pendingHealthchecks.length }} service{{
              pendingHealthchecks.length > 1 ? 's' : ''
            }}
            pending:</span
          >
          <div class="flex flex-wrap gap-1 mt-1">
            <router-link
              v-for="item in pendingHealthchecks"
              :key="item.healthcheck.id"
              :to="healthcheckTarget(item.healthcheck)"
              class="badge badge-error gap-1"
            >
              {{ item.healthcheck.name }}
            </router-link>
          </div>
        </div>
      </div>

      <!-- Stat Cards Row -->
      <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4 mb-8">
        <router-link
          to="/applications"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Applications</div>
          <div class="stat-value text-primary">
            {{ stats.applications.total }}
          </div>
          <div class="stat-desc">{{ stats.applications.active }} active</div>
        </router-link>

        <router-link
          to="/services"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Services</div>
          <div class="stat-value text-secondary">
            {{ stats.services.total }}
          </div>
          <div class="stat-desc">{{ stats.services.active }} active</div>
        </router-link>

        <router-link
          to="/infra"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Infrastructure</div>
          <div class="stat-value text-accent">{{ stats.infra.total }}</div>
          <div class="stat-desc">{{ stats.infra.active }} active</div>
        </router-link>

        <router-link
          to="/domains"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Domains</div>
          <div class="stat-value text-info">{{ stats.domains.total }}</div>
          <div class="stat-desc">{{ stats.domains.active }} active</div>
        </router-link>

        <router-link
          to="/people"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">People</div>
          <div class="stat-value text-warning">{{ stats.people.total }}</div>
          <div class="stat-desc">{{ stats.people.active }} active</div>
        </router-link>

        <router-link
          to="/shares"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Storage</div>
          <div class="stat-value text-error">
            {{ stats.network_shares.total }}
          </div>
          <div class="stat-desc">{{ stats.network_shares.active }} active</div>
        </router-link>

        <router-link
          to="/healthchecks"
          class="stat bg-base-200 rounded-lg hover:bg-base-300 transition-colors"
        >
          <div class="stat-title">Healthchecks</div>
          <div class="stat-value text-success">
            {{ stats.healthchecks.total }}
          </div>
          <div class="stat-desc">{{ stats.healthchecks.enabled }} enabled</div>
        </router-link>
      </div>

      <!-- Service Health Grid -->
      <div v-if="healthcheckStatuses.length > 0" class="card bg-base-200 mb-8">
        <div class="card-body">
          <h2 class="card-title">Health</h2>
          <div
            class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-x-8 gap-y-1"
          >
            <div
              v-for="item in healthcheckStatuses"
              :key="item.healthcheck.id"
              class="grid grid-cols-3 gap-2 py-2 border-b border-base-300"
            >
              <!-- Status dot -->
              <div class="flex items-center justify-end gap-2 overflow-hidden">
                <span
                  class="w-2.5 h-2.5 rounded-full shrink-0"
                  :class="statusDotClass(item.status)"
                />
                <router-link
                  :to="healthcheckTarget(item.healthcheck)"
                  class="link link-hover min-w-0 text-right marquee-container"
                >
                  <span class="marquee-text">{{
                    toTitleCase(item.healthcheck.name)
                  }}</span>
                </router-link>
              </div>

              <!-- Sparkline -->
              <div
                v-if="item.healthcheck.kuma_id"
                class="col-span-2 max-w-100 md:max-w-50 lg:max-w-80 xl:max-w-80 ml-1 h-4 shrink-0 hidden sm:block"
              >
                <HealthPlot :kuma-id="item.healthcheck.kuma_id" :count="48" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Two-column: Expiring Domains + Recent Activity -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Expiring Domains -->
        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title text-warning">Expiring Domains (90 days)</h2>
            <div
              v-if="stats.expiring_domains.length === 0"
              class="text-base-content/70"
            >
              No domains expiring soon
            </div>
            <ul v-else class="space-y-2">
              <li
                v-for="d in stats.expiring_domains"
                :key="d.id"
                class="flex justify-between items-center"
              >
                <router-link :to="`/domains/${d.id}`" class="link link-hover">{{
                  d.name
                }}</router-link>
                <span class="badge badge-warning">{{
                  formatDate(d.expires_at)
                }}</span>
              </li>
            </ul>
          </div>
        </div>

        <!-- Recent Activity -->
        <div class="card bg-base-200">
          <div class="card-body">
            <h2 class="card-title">Recent Changes</h2>
            <div
              v-if="stats.recent_activity.length === 0"
              class="text-base-content/70"
            >
              No recent activity
            </div>
            <ul v-else class="space-y-2">
              <li
                v-for="item in stats.recent_activity"
                :key="item.id + item.entity_type"
                class="flex items-center gap-2"
              >
                <span
                  class="badge badge-sm shrink-0"
                  :class="entityBadgeClass(item.entity_type)"
                  >{{ entityLabel(item.entity_type) }}</span
                >
                <router-link
                  :to="entityRoute(item)"
                  class="link link-hover truncate min-w-0 flex-1"
                  >{{ item.name }}</router-link
                >
                <span class="text-xs text-base-content/50 shrink-0">{{
                  formatRelativeTime(item.updated_at)
                }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.marquee-container {
  overflow: hidden;
  display: inline-block;
  max-width: 100%;
}

.marquee-text {
  display: inline-block;
  white-space: nowrap;
}

.marquee-container:hover .marquee-text {
  animation: marquee-scroll 4s linear infinite;
}

@keyframes marquee-scroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(-100%);
  }
}
</style>
