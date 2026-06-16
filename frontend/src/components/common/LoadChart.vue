<script setup lang="ts">
import { computed } from 'vue';
import type { LoadPoint } from '@/types';

const props = withDefaults(
  defineProps<{ points: LoadPoint[]; height?: number }>(),
  { height: 140 }
);

// Fixed viewBox width; the SVG scales to its container (preserveAspectRatio=none),
// and non-scaling-stroke keeps line thickness constant despite the x-stretch.
const W = 600;

const metrics = [
  { key: 'cpu' as const, label: 'CPU', color: 'text-info' },
  { key: 'mem' as const, label: 'Memory', color: 'text-warning' },
  { key: 'swap' as const, label: 'Swap', color: 'text-secondary' },
];

const tRange = computed<[number, number]>(() => {
  const ts = props.points.map((p) => p.t);
  return ts.length ? [Math.min(...ts), Math.max(...ts)] : [0, 1];
});

// Each metric is its own polyline over its non-null samples; Zabbix stores the
// three metrics at distinct clocks, so they don't share x-positions.
function path(key: 'cpu' | 'mem' | 'swap'): string {
  const [t0, t1] = tRange.value;
  const span = t1 - t0 || 1;
  const h = props.height;
  const pts = props.points
    .filter((p) => p[key] != null)
    .map((p) => {
      const x = ((p.t - t0) / span) * W;
      const y = h - (Math.min(100, p[key]!) / 100) * h;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    });
  return pts.length ? 'M' + pts.join(' L') : '';
}

// Latest non-null value per metric, for the legend.
function current(key: 'cpu' | 'mem' | 'swap'): number | null {
  for (let i = props.points.length - 1; i >= 0; i--) {
    const v = props.points[i][key];
    if (v != null) return v;
  }
  return null;
}
function fmt(v: number | null): string {
  return v == null ? '–' : `${Math.round(v)}%`;
}
</script>

<template>
  <div>
    <!-- chart + y-axis labels overlaid (text mustn't be x-stretched) -->
    <div class="relative">
      <svg
        :viewBox="`0 0 ${W} ${height}`"
        preserveAspectRatio="none"
        class="w-full block"
        :style="{ height: `${height}px` }"
      >
        <line
          v-for="g in [0, 50, 100]"
          :key="g"
          x1="0"
          :x2="W"
          :y1="height - (g / 100) * height"
          :y2="height - (g / 100) * height"
          class="stroke-base-300"
          stroke-width="1"
        />
        <path
          v-for="m in metrics"
          :key="m.key"
          :d="path(m.key)"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="[vector-effect:non-scaling-stroke]"
          :class="m.color"
        />
      </svg>
      <!-- y-axis tick labels -->
      <span
        v-for="g in [0, 50, 100]"
        :key="g"
        class="absolute left-0 -translate-y-1/2 text-[10px] text-base-content/50 bg-base-200 pr-1 tabular-nums"
        :style="{ top: `${height - (g / 100) * height}px` }"
        >{{ g }}%</span
      >
    </div>
    <div class="flex flex-wrap gap-4 mt-1 text-xs">
      <span
        v-for="m in metrics"
        :key="m.key"
        class="flex items-center gap-1"
        :class="m.color"
      >
        <span
          class="inline-block w-3 h-0.5"
          :class="m.color"
          style="background: currentColor"
        />
        {{ m.label }}
        <span class="tabular-nums font-medium">{{ fmt(current(m.key)) }}</span>
      </span>
    </div>
  </div>
</template>
