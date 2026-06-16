<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import type { InfraLoad } from '@/types';

const props = withDefaults(
  defineProps<{
    load: InfraLoad;
    cpu?: boolean;
    mem?: boolean;
    swap?: boolean;
    orientation?: 'vertical' | 'horizontal';
    numbers?: boolean;
  }>(),
  { cpu: true, mem: true, swap: true, orientation: 'vertical', numbers: false }
);

// Thresholds shared with the graph view: green < 70, amber < 90, red ≥ 90.
function barClass(v: number | null): string {
  if (v == null) return 'bg-base-300';
  if (v < 70) return 'bg-success';
  if (v < 90) return 'bg-warning';
  return 'bg-error';
}

const metrics = computed(() =>
  (
    [
      { key: 'cpu', label: 'CPU', on: props.cpu },
      { key: 'mem', label: 'MEM', on: props.mem },
      { key: 'swap', label: 'SWP', on: props.swap },
    ] as const
  ).filter((m) => m.on)
);

// Grow bars from 0 to their value once mounted (data arrives async).
const shown = ref(false);
onMounted(() => requestAnimationFrame(() => (shown.value = true)));
function pct(v: number | null): number {
  return shown.value ? Math.min(100, v ?? 0) : 0;
}
function label(v: number | null): string {
  return v == null ? '–' : `${Math.round(v)}%`;
}
</script>

<template>
  <!-- Vertical: stacked label + horizontal progress bar (detail view). -->
  <div v-if="orientation === 'vertical'" class="flex flex-col gap-0.5 w-28">
    <div
      v-for="m in metrics"
      :key="m.key"
      class="flex items-center gap-1"
      :title="`${m.label}: ${label(load[m.key])}`"
    >
      <span class="text-[10px] w-7 text-base-content/60">{{ m.label }}</span>
      <div class="flex-1 h-1.5 bg-base-300 rounded-full overflow-hidden">
        <div
          class="h-full rounded-full transition-all duration-700 ease-out"
          :class="barClass(load[m.key])"
          :style="{ width: `${pct(load[m.key])}%` }"
        />
      </div>
      <span v-if="numbers" class="text-[10px] w-8 text-right tabular-nums">
        {{ label(load[m.key]) }}
      </span>
    </div>
  </div>

  <!-- Horizontal: metrics side by side on one line (list view, keeps rows short). -->
  <div v-else class="flex items-center gap-2">
    <div
      v-for="m in metrics"
      :key="m.key"
      class="flex items-center gap-1"
      :title="`${m.label}: ${label(load[m.key])}`"
    >
      <span class="text-[10px] text-base-content/60">{{ m.label }}</span>
      <div class="w-8 h-1.5 bg-base-300 rounded-full overflow-hidden">
        <div
          class="h-full rounded-full transition-all duration-700 ease-out"
          :class="barClass(load[m.key])"
          :style="{ width: `${pct(load[m.key])}%` }"
        />
      </div>
      <span
        v-if="numbers"
        class="text-[10px] tabular-nums w-7 text-right"
        >{{ label(load[m.key]) }}</span
      >
    </div>
  </div>
</template>
