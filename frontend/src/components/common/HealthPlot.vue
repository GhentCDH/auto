<script setup lang="ts">
import { computed } from 'vue';
import { useUptime } from '@/composables/useUptime';

const props = withDefaults(
  defineProps<{
    kumaId: number;
    count?: number;
  }>(),
  { count: 45 }
);

const { getMonitorData, monitors } = useUptime();

const bars = computed(() => {
  // Access monitors.value to ensure reactivity triggers
  const _ = monitors.value;
  const data = getMonitorData(props.kumaId);
  if (!data) return [];
  return data.heartbeats.slice(-props.count);
});

function barColorClass(status: number): string {
  if (status === 1) return 'bg-success'; // green-500
  if (status === 0) return 'bg-error'; // red-500
  return 'bg-neutral-content'; // gray-500 (pending/maintenance)
}

function formatHumanTime(input: string): string {
  // Normalize to proper ISO format
  const iso = input.replace(' ', 'T');

  const date = new Date(iso);

  if (isNaN(date.getTime())) {
    return 'Invalid date';
  }

  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}
</script>

<template>
  <div
    class="flex items-end gap-px h-full w-full"
    :title="`Monitor #${kumaId}`"
  >
    <!-- Placeholder bars when no data yet -->
    <template v-if="bars.length === 0">
      <div
        v-for="i in count"
        :key="i"
        class="flex-1 rounded-sm bg-base-300 h-full opacity-30"
      />
    </template>

    <template v-else>
      <!-- Spacer when fewer than `count` heartbeats exist -->
      <div
        v-if="bars.length < count"
        v-for="i in count - bars.length"
        :key="-i"
        class="flex-1 rounded-sm min-w-px bg-base-300 opacity-30 h-full"
      />
      <div
        v-for="(entry, i) in bars"
        :key="i"
        class="flex-1 rounded-sm min-w-px"
        :class="barColorClass(entry.status)"
        :style="{
          height: '100%',
        }"
        :title="`${formatHumanTime(entry.time)} — ${entry.status === 1 ? 'UP' : 'DOWN'}${entry.ping ? ` (${entry.ping}ms)` : ''}${entry.msg ? `: ${entry.msg}` : ''}`"
      />
    </template>
  </div>
</template>
