<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { dashboardApi } from '@/api';
import type { DashboardStats } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';

const stats = ref<DashboardStats | null>(null);
const loading = ref(true);
const error = ref('');

onMounted(async () => {
  try {
    stats.value = await dashboardApi.getStats();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load dashboard';
  } finally {
    loading.value = false;
  }
});

function formatDate(date: string | null): string {
  if (!date) return 'N/A';
  return new Date(date).toLocaleDateString();
}
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">Dashboard</h1>

    <LoadingSpinner v-if="loading" />

    <div v-else-if="error" class="alert alert-error">{{ error }}</div>

    <div v-else-if="stats">
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4 mb-8">
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
          <div class="stat-value text-secondary">{{ stats.services.total }}</div>
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
          <div class="stat-title">Shares</div>
          <div class="stat-value text-error">
            {{ stats.network_shares.total }}
          </div>
          <div class="stat-desc">{{ stats.network_shares.active }} active</div>
        </router-link>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
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
      </div>
    </div>
  </div>
</template>
