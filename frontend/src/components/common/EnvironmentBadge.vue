<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  environment: string;
}>();

const badgeClass = computed(() => {
  switch (props.environment.toLowerCase()) {
    case 'dev':
      return 'badge-info';
    case 'qas':
      return 'badge-warning';
    case 'prd':
      return 'badge-success';
    case 'tst':
      return 'badge-accent';
    default:
      return 'badge-neutral';
  }
});
</script>

<template>
  <span class="badge badge-sm" :class="badgeClass">{{ environment }}</span>
</template>
