<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { searchApi } from '@/api';
import type { SearchResults, SearchResult } from '@/types';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';
import EmptyState from '@/components/common/EmptyState.vue';

const route = useRoute();
const router = useRouter();

const results = ref<SearchResults | null>(null);
const loading = ref(false);
const error = ref('');
const query = ref((route.query.q as string) || '');
const routes: Record<string, string> = {
  application: 'applications',
  service: 'services',
  infra: 'infra',
  domain: 'domains',
  person: 'people',
  network_share: 'shares',
  stack: 'stack',
  healthcheck: 'healthchecks',
};

const totalResults = computed(() => {
  if (!results.value) return 0;
  return (
    results.value.applications.length +
    results.value.services.length +
    results.value.infra.length +
    results.value.domains.length +
    results.value.people.length +
    results.value.network_shares.length +
    results.value.stacks.length +
    results.value.healthchecks.length
  );
});

async function search() {
  if (!query.value.trim()) {
    results.value = null;
    return;
  }

  loading.value = true;
  error.value = '';
  try {
    results.value = await searchApi.search(query.value);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Search failed';
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  router.push({ query: { q: query.value } });
}

function navigateTo(result: SearchResult) {
  const path = routes[result.entity_type] || result.entity_type;
  router.push(`/${path}/${result.id}`);
}

watch(
  () => route.query.q,
  () => {
    query.value = (route.query.q as string) || '';
    search();
  },
  { immediate: true }
);

onMounted(search);
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">Search</h1>

    <form @submit.prevent="handleSearch" class="mb-6">
      <div class="join w-full max-w-xl">
        <input
          v-model="query"
          type="text"
          placeholder="Search applications, services, infra, domains, people..."
          class="input input-bordered join-item flex-1"
          autofocus
        />
        <button type="submit" class="btn btn-primary join-item">Search</button>
      </div>
    </form>

    <LoadingSpinner v-if="loading" />
    <div v-else-if="error" class="alert alert-error">{{ error }}</div>
    <EmptyState
      v-else-if="query && totalResults === 0"
      message="No results found"
    />

    <div v-else-if="results && totalResults > 0" class="space-y-6">
      <p class="text-base-content/70">
        Found {{ totalResults }} results for "{{ route.query.q }}"
      </p>

      <div v-if="results.applications.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">
            Applications ({{ results.applications.length }})
          </h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.applications"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div v-if="r.description" class="text-sm text-base-content/70">
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.services.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">Services ({{ results.services.length }})</h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.services"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div v-if="r.description" class="text-sm text-base-content/70">
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.infra.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">
            Infrastructure ({{ results.infra.length }})
          </h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.infra"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div v-if="r.description" class="text-sm text-base-content/70">
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.domains.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">Domains ({{ results.domains.length }})</h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.domains"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div v-if="r.description" class="text-sm text-base-content/70">
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.people.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">People ({{ results.people.length }})</h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.people"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div v-if="r.description" class="text-sm text-base-content/70">
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.network_shares.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">
            Storage ({{ results.network_shares.length }})
          </h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.network_shares"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div
                v-if="r.description"
                class="text-sm text-base-content/70 font-mono"
              >
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.stacks.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">Technologies ({{ results.stacks.length }})</h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.stacks"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div
                v-if="r.description"
                class="text-sm text-base-content/70 font-mono"
              >
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>

      <div v-if="results.healthchecks.length" class="card bg-base-200">
        <div class="card-body">
          <h2 class="card-title">
            Healthchecks ({{ results.healthchecks.length }})
          </h2>
          <ul class="space-y-2">
            <li
              v-for="r in results.healthchecks"
              :key="r.id"
              class="cursor-pointer hover:bg-base-300 p-2 rounded"
              @click="navigateTo(r)"
            >
              <div class="font-medium">{{ r.name }}</div>
              <div
                v-if="r.description"
                class="text-sm text-base-content/70 font-mono"
              >
                {{ r.description }}
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
