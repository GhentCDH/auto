<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  status: string;
}>();

const badgeClass = computed(() => {
  switch (props.status.toLowerCase()) {
    case 'active':
      return 'badge-success';
    case 'inactive':
    case 'deprecated':
      return 'badge-warning';
    case 'archived':
    case 'deleted':
      return 'badge-error';
    default:
      return 'badge-neutral';
  }
});
</script>

<template>
  <span class="badge badge-sm" :class="badgeClass">{{ status }}</span>
</template>
