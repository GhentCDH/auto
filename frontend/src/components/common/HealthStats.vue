<script setup lang="ts">
import { computed } from 'vue';
import { useUptime } from '@/composables/useUptime';

const props = defineProps<{
  kumaId?: number;
  kumaIds?: number[];
}>();

const { getMonitorData, monitors } = useUptime();

const ids = computed(() => {
  if (props.kumaIds !== undefined) return props.kumaIds;
  if (props.kumaId !== undefined) return [props.kumaId];
  return [];
});

const stats = computed(() => {
  const _ = monitors.value;
  const monitorIds = ids.value;
  if (monitorIds.length === 0) return null;

  let total = 0;
  let up = 0;
  for (const id of monitorIds) {
    const data = getMonitorData(id);
    if (!data || data.heartbeats.length === 0) continue;
    total += data.heartbeats.length;
    up += data.heartbeats.filter((h) => h.status === 1).length;
  }
  if (total === 0) return null;

  const pct = Math.round((up / total) * 1000) / 10;
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
