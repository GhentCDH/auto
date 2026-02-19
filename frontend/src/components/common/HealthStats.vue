<script setup lang="ts">
import { computed } from 'vue';
import { useUptime } from '@/composables/useUptime';

const props = defineProps<{
  kumaId: number;
}>();

const { getMonitorData, monitors } = useUptime();

const stats = computed(() => {
  // Access monitors.value to ensure reactivity triggers
  const _ = monitors.value;
  const data = getMonitorData(props.kumaId);
  if (!data || data.heartbeats.length === 0) return null;

  const total = data.heartbeats.length;
  const up = data.heartbeats.filter((h) => h.status === 1).length;
  const pct = Math.round((up / total) * 1000) / 10; // 1 decimal place

  return { pct };
});

const badgeClass = computed(() => {
  if (!stats.value) return 'badge-neutral';
  if (stats.value.pct >= 99) return 'badge-success';
  if (stats.value.pct >= 95) return 'badge-warning';
  return 'badge-error';
});
</script>

<template>
  <span v-if="stats" class="badge badge-sm font-mono" :class="badgeClass">
    {{ stats.pct }}%
  </span>
  <span v-else class="badge badge-sm badge-ghost opacity-50">&mdash;</span>
</template>
