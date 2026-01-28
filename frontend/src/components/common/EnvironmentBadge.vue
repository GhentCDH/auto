<script setup lang="ts">
import { computed } from 'vue';
import { environments } from '@/values';

const props = defineProps<{
  environment: string;
}>();

const badgeClass = computed(() => {
  switch (props.environment.toLowerCase()) {
    case 'dev':
      return 'badge-info';
    case 'staging':
      return 'badge-warning';
    case 'prd':
      return 'badge-success';
    default:
      return 'badge-neutral';
  }
});

const label = computed(() => {
  return environments[props.environment as keyof typeof environments] || props.environment;
});
</script>

<template>
  <span class="badge badge-sm" :class="badgeClass">{{ label }}</span>
</template>
