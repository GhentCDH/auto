<script setup lang="ts">
import { onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { resolveApi } from '@/api';

const route = useRoute();
const router = useRouter();

/** Maps backend entity_type to frontend route prefix */
const ENTITY_ROUTES: Record<string, string> = {
  application: '/applications',
  service: '/services',
  infra: '/infra',
  domain: '/domains',
  person: '/people',
  network_share: '/shares',
  stack: '/stack',
  healthcheck: '/healthchecks',
};

onMounted(async () => {
  try {
    const resolved = await resolveApi.resolve(route.params.id as string);
    const prefix = ENTITY_ROUTES[resolved.entity_type];
    if (prefix) {
      router.replace(`${prefix}/${resolved.id}`);
    } else {
      router.replace('/');
    }
  } catch {
    router.replace('/');
  }
});
</script>

<template>
  <div class="flex items-center justify-center min-h-48">
    <span class="loading loading-spinner loading-lg"></span>
  </div>
</template>
