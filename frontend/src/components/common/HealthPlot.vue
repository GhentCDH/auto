<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useUptime } from '@/composables/useUptime';

const props = withDefaults(
  defineProps<{
    kumaId: number;
    count?: number;
    // Width (px) of a single healthcheck instant. When set, the number of
    // instants shown adapts to the available width while each tick keeps this
    // fixed width. When undefined, falls back to a fixed `count` of stretched bars.
    tickWidth?: number;
    // Optional clamps on the adaptive instant count (only used with tickWidth).
    minTicks?: number;
    maxTicks?: number;
  }>(),
  { count: 45, tickWidth: undefined, minTicks: undefined, maxTicks: undefined }
);

const { getMonitorData, monitors } = useUptime();

const containerRef = ref<HTMLElement | null>(null);
const containerWidth = ref(0);

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  if (!containerRef.value) return;
  resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      containerWidth.value = entry.contentRect.width;
    }
  });
  resizeObserver.observe(containerRef.value);
  containerWidth.value = containerRef.value.clientWidth;
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
});

// gap-px between bars is 1px each.
const GAP = 1;

const effectiveCount = computed(() => {
  if (props.tickWidth === undefined) return props.count;
  if (containerWidth.value <= 0) return 0;
  const slot = props.tickWidth + GAP;
  let n = Math.max(1, Math.floor((containerWidth.value + GAP) / slot));
  if (props.minTicks !== undefined) n = Math.max(n, props.minTicks);
  if (props.maxTicks !== undefined) n = Math.min(n, props.maxTicks);
  return n;
});

const barStyle = computed(() => {
  if (props.tickWidth === undefined) return undefined;
  return { width: `${props.tickWidth}px`, flex: 'none' };
});

const bars = computed(() => {
  // Access monitors.value to ensure reactivity triggers
  const _ = monitors.value;
  const data = getMonitorData(props.kumaId);
  if (!data) return [];
  return data.heartbeats.slice(-effectiveCount.value);
});

function barColorClass(status: number): string {
  if (status === 1) return 'bg-success';
  if (status === 2) return 'bg-warning'; // green-500
  if (status === 0) return 'bg-error';
  return 'bg-neutral-content'; // gray-500 (maintenance)
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
    ref="containerRef"
    class="flex items-end gap-px h-full w-full overflow-hidden"
    :class="{ 'justify-end': tickWidth !== undefined }"
    :title="`Monitor #${kumaId}`"
  >
    <!-- Placeholder bars when no data yet -->
    <template v-if="bars.length === 0">
      <div
        v-for="i in effectiveCount"
        :key="i"
        class="flex-1 rounded-sm bg-base-300 h-full opacity-30"
        :style="barStyle"
      />
    </template>

    <template v-else>
      <!-- Spacer when fewer than `effectiveCount` heartbeats exist -->
      <div
        v-if="bars.length < effectiveCount"
        v-for="i in effectiveCount - bars.length"
        :key="-i"
        class="flex-1 rounded-sm min-w-px bg-base-300 opacity-30 h-full"
        :style="barStyle"
      />
      <div
        v-for="(entry, i) in bars"
        :key="i"
        class="flex-1 rounded-sm min-w-px"
        :class="barColorClass(entry.status)"
        :style="{
          height: '100%',
          ...barStyle,
        }"
        :title="`${formatHumanTime(entry.time)} — ${entry.status === 1 ? 'UP' : 'DOWN'}${entry.ping ? ` (${entry.ping}ms)` : ''}${entry.msg ? `: ${entry.msg}` : ''}`"
      />
    </template>
  </div>
</template>
