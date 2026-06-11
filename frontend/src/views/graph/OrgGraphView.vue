<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { RotateCw, Crosshair } from 'lucide-vue-next';
import { useOrgGraph, uptimePct, type GraphNodeType } from '@/composables/useOrgGraph';
import { useForceGraph, type SimNode } from '@/composables/useForceGraph';
import { useUptime } from '@/composables/useUptime';
import HealthPlot from '@/components/common/HealthPlot.vue';

const router = useRouter();
const { data, loading, error, progress, refresh } = useOrgGraph();
const { monitors } = useUptime();

const containerRef = ref<HTMLElement | null>(null);
const svgRef = ref<SVGSVGElement | null>(null);

const force = useForceGraph(1200, 800);
const { simNodes, simEdges, tick, setData, resize, reheat, dragStart, drag, dragEnd } =
  force;

// ---- type filters -------------------------------------------------------
const typeEnabled = reactive<Record<GraphNodeType, boolean>>({
  application: true,
  service: true,
  infra: true,
});

const visible = computed(() => {
  const nodes = data.value.nodes.filter((n) => typeEnabled[n.type]);
  const ids = new Set(nodes.map((n) => n.id));
  const edges = data.value.edges.filter(
    (e) => ids.has(e.source) && ids.has(e.target)
  );
  return { nodes, edges };
});

watch(
  visible,
  (v) => {
    setData(v.nodes, v.edges);
  },
  { deep: false }
);

// ---- health colors ------------------------------------------------------
function nodePct(node: SimNode): number | null {
  // touch tick + monitors so colors stay reactive to live heartbeats
  void monitors.value;
  return uptimePct(node.kumaIds, monitors.value);
}

function fillClass(node: SimNode): string {
  const pct = nodePct(node);
  if (pct === null) return 'fill-base-300';
  if (pct >= 99) return 'fill-success';
  if (pct >= 95) return 'fill-warning';
  return 'fill-error';
}


// ---- adjacency / highlight ---------------------------------------------
const adjacency = computed(() => {
  const map = new Map<string, Set<string>>();
  for (const n of visible.value.nodes) map.set(n.id, new Set());
  for (const e of visible.value.edges) {
    map.get(e.source)?.add(e.target);
    map.get(e.target)?.add(e.source);
  }
  return map;
});

const hoveredId = ref<string | null>(null);
const pinnedId = ref<string | null>(null);

const focusId = computed(() => pinnedId.value ?? hoveredId.value);
const highlightSet = computed(() => {
  const id = focusId.value;
  if (!id) return null;
  const set = new Set<string>([id]);
  for (const nb of adjacency.value.get(id) ?? []) set.add(nb);
  return set;
});

function nodeDimmed(id: string): boolean {
  return highlightSet.value !== null && !highlightSet.value.has(id);
}
function edgeActive(e: { source: string; target: string }): boolean {
  const id = focusId.value;
  return !!id && (e.source === id || e.target === id);
}

// ---- pan / zoom ---------------------------------------------------------
const view = reactive({ x: 0, y: 0, k: 1 });
const transform = computed(() => `translate(${view.x},${view.y}) scale(${view.k})`);

function svgPoint(clientX: number, clientY: number) {
  const rect = svgRef.value!.getBoundingClientRect();
  return { x: clientX - rect.left, y: clientY - rect.top };
}
function toWorld(clientX: number, clientY: number) {
  const p = svgPoint(clientX, clientY);
  return { x: (p.x - view.x) / view.k, y: (p.y - view.y) / view.k };
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const p = svgPoint(e.clientX, e.clientY);
  const world = { x: (p.x - view.x) / view.k, y: (p.y - view.y) / view.k };
  const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
  const k = Math.min(Math.max(view.k * factor, 0.2), 4);
  view.x = p.x - world.x * k;
  view.y = p.y - world.y * k;
  view.k = k;
}

// ---- dragging -----------------------------------------------------------
type DragState =
  | { mode: 'none' }
  | { mode: 'pan'; startX: number; startY: number; ox: number; oy: number }
  | { mode: 'node'; node: SimNode; moved: boolean };
let dragState: DragState = { mode: 'none' };
const panning = ref(false);

function onBgPointerDown(e: PointerEvent) {
  pinnedId.value = null;
  dragState = { mode: 'pan', startX: e.clientX, startY: e.clientY, ox: view.x, oy: view.y };
  panning.value = true;
  window.addEventListener('pointermove', onPointerMove);
  window.addEventListener('pointerup', onPointerUp);
}

function onNodePointerDown(e: PointerEvent, node: SimNode) {
  e.stopPropagation();
  dragState = { mode: 'node', node, moved: false };
  dragStart(node);
  window.addEventListener('pointermove', onPointerMove);
  window.addEventListener('pointerup', onPointerUp);
}

function onPointerMove(e: PointerEvent) {
  if (dragState.mode === 'pan') {
    view.x = dragState.ox + (e.clientX - dragState.startX);
    view.y = dragState.oy + (e.clientY - dragState.startY);
  } else if (dragState.mode === 'node') {
    const w = toWorld(e.clientX, e.clientY);
    drag(dragState.node, w.x, w.y);
    dragState.moved = true;
  }
}

function onPointerUp() {
  if (dragState.mode === 'node') {
    // Keep node pinned where dropped if it was actually moved.
    dragEnd(dragState.node, dragState.moved);
    if (!dragState.moved) {
      pinnedId.value = pinnedId.value === dragState.node.id ? null : dragState.node.id;
    }
  }
  dragState = { mode: 'none' };
  panning.value = false;
  window.removeEventListener('pointermove', onPointerMove);
  window.removeEventListener('pointerup', onPointerUp);
}

function openNode(node: SimNode) {
  const path =
    node.type === 'application'
      ? '/applications/'
      : node.type === 'service'
        ? '/services/'
        : '/infra/';
  router.push(path + node.id);
}

// ---- popover ------------------------------------------------------------
const hoveredNode = computed(
  () => simNodes.value.find((n) => n.id === hoveredId.value) ?? null
);
const popoverStyle = computed(() => {
  void tick.value;
  const n = hoveredNode.value;
  if (!n || n.x == null || n.y == null) return { display: 'none' };
  return {
    left: `${view.x + n.x * view.k}px`,
    top: `${view.y + n.y * view.k - (n.radius * view.k + 12)}px`,
  };
});

// ---- layout sizing ------------------------------------------------------
let ro: ResizeObserver | null = null;
function recenter() {
  const el = containerRef.value;
  if (!el) return;
  view.x = 0;
  view.y = 0;
  view.k = 1;
  resize(el.clientWidth, el.clientHeight);
}

onMounted(async () => {
  const el = containerRef.value!;
  resize(el.clientWidth, el.clientHeight);
  ro = new ResizeObserver(() => resize(el.clientWidth, el.clientHeight));
  ro.observe(el);
  await refresh();
});
onBeforeUnmount(() => ro?.disconnect());
</script>

<template>
  <div class="container mx-auto p-4">
    <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
      <div>
        <h1 class="text-2xl font-bold">Org Graph</h1>
        <p class="text-sm opacity-60">
          Applications, services & infra and how they connect — coloured by uptime.
        </p>
      </div>
      <div class="flex items-center gap-2">
        <label
          v-for="t in (['application', 'service', 'infra'] as GraphNodeType[])"
          :key="t"
          class="label cursor-pointer gap-1 text-sm capitalize"
        >
          <input
            type="checkbox"
            class="checkbox checkbox-sm"
            v-model="typeEnabled[t]"
          />
          {{ t }}
        </label>
        <button class="btn btn-sm btn-ghost" @click="recenter" title="Recenter">
          <Crosshair class="w-4 h-4" /> Fit
        </button>
        <button class="btn btn-sm" :disabled="loading" @click="refresh">
          <RotateCw class="w-4 h-4" :class="{ 'animate-spin': loading }" /> Reload
        </button>
      </div>
    </div>

    <div
      v-if="error"
      class="alert alert-error mb-4"
    >
      <span>{{ error }}</span>
    </div>

    <div
      ref="containerRef"
      class="relative w-full rounded-box border border-base-300 bg-base-100 overflow-hidden"
      style="height: calc(100vh - 12rem)"
    >
      <!-- loading overlay -->
      <div
        v-if="loading"
        class="absolute inset-0 z-20 flex flex-col items-center justify-center gap-3 bg-base-100/70"
      >
        <span class="loading loading-spinner loading-lg"></span>
        <span v-if="progress" class="text-sm font-mono opacity-70">
          loading relations {{ progress.done }}/{{ progress.total }}
        </span>
      </div>

      <svg
        ref="svgRef"
        class="w-full h-full select-none"
        :class="panning ? 'cursor-grabbing' : 'cursor-grab'"
        @pointerdown="onBgPointerDown"
        @wheel="onWheel"
      >
        <g :transform="transform">
          <!-- edges -->
          <line
            v-for="(e, i) in simEdges"
            :key="'e' + i"
            :x1="e.source.x"
            :y1="e.source.y"
            :x2="e.target.x"
            :y2="e.target.y"
            :class="
              edgeActive({ source: e.source.id, target: e.target.id })
                ? 'stroke-base-content'
                : 'stroke-base-300'
            "
            :stroke-width="edgeActive({ source: e.source.id, target: e.target.id }) ? 2 : 1"
            :stroke-opacity="
              highlightSet && !edgeActive({ source: e.source.id, target: e.target.id })
                ? 0.15
                : 0.6
            "
            :data-tick="tick"
          />

          <!-- nodes -->
          <g
            v-for="n in simNodes"
            :key="n.id"
            :transform="`translate(${n.x ?? 0},${n.y ?? 0})`"
            :opacity="nodeDimmed(n.id) ? 0.2 : 1"
            class="cursor-pointer"
            :data-tick="tick"
            @pointerdown="onNodePointerDown($event, n)"
            @pointerenter="hoveredId = n.id"
            @pointerleave="hoveredId = null"
            @dblclick="openNode(n)"
          >
<!-- application = circle -->
            <circle
              v-if="n.type === 'application'"
              :r="n.radius"
              :class="fillClass(n)"
            />
            <!-- infra = square -->
            <rect
              v-else-if="n.type === 'infra'"
              :x="-n.radius"
              :y="-n.radius"
              :width="n.radius * 2"
              :height="n.radius * 2"
              rx="2"
              :class="fillClass(n)"
            />
            <!-- service = diamond (45°-rotated square) -->
            <rect
              v-else
              :x="-n.radius * 0.88"
              :y="-n.radius * 0.88"
              :width="n.radius * 1.76"
              :height="n.radius * 1.76"
              rx="2"
              transform="rotate(45)"
              :class="fillClass(n)"
            />
            <text
              text-anchor="middle"
              :y="n.radius + 12"
              class="fill-base-content pointer-events-none"
              font-size="10"
            >
              {{ n.name.length > 22 ? n.name.slice(0, 21) + '…' : n.name }}
            </text>
          </g>
        </g>
      </svg>

      <!-- hover popover -->
      <div
        v-if="hoveredNode"
        class="absolute z-30 -translate-x-1/2 -translate-y-full pointer-events-none"
        :style="popoverStyle"
      >
        <div class="card bg-base-200 shadow-lg border border-base-300 w-64">
          <div class="card-body p-3 gap-1">
            <div class="flex items-center justify-between gap-2">
              <span class="font-bold truncate">{{ hoveredNode.name }}</span>
              <span class="badge badge-sm capitalize">{{ hoveredNode.type }}</span>
            </div>
            <div class="text-xs opacity-70 flex flex-wrap gap-x-3">
              <span v-if="hoveredNode.environment">env: {{ hoveredNode.environment }}</span>
              <span v-if="hoveredNode.status">{{ hoveredNode.status }}</span>
              <span>{{ hoveredNode.degree }} link{{ hoveredNode.degree === 1 ? '' : 's' }}</span>
            </div>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs opacity-70">uptime</span>
              <span
                v-if="nodePct(hoveredNode) !== null"
                class="badge badge-sm font-mono"
                :class="
                  nodePct(hoveredNode)! >= 99
                    ? 'badge-success'
                    : nodePct(hoveredNode)! >= 95
                      ? 'badge-warning'
                      : 'badge-error'
                "
              >
                {{ nodePct(hoveredNode) }}%
              </span>
              <span v-else class="badge badge-sm badge-ghost opacity-50">no monitor</span>
            </div>
            <div v-if="hoveredNode.kumaIds.length === 1" class="h-6 mt-1">
              <HealthPlot :kuma-id="hoveredNode.kumaIds[0]" :tick-width="5" :max-ticks="48" />
            </div>
            <div class="text-[10px] opacity-50 mt-1">
              click to focus · double-click to open
            </div>
          </div>
        </div>
      </div>

      <!-- legend -->
      <div
        class="absolute bottom-3 left-3 z-10 bg-base-200/80 rounded-box p-2 text-xs space-y-1 pointer-events-none"
      >
        <div class="font-semibold opacity-70">uptime</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rounded-full bg-success inline-block"></span> ≥99%</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rounded-full bg-warning inline-block"></span> ≥95%</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rounded-full bg-error inline-block"></span> &lt;95%</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rounded-full bg-base-300 inline-block"></span> no monitor</div>
        <div class="font-semibold opacity-70 pt-1">type</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rounded-full bg-base-content inline-block"></span> application</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 rotate-45 bg-base-content inline-block"></span> service</div>
        <div class="flex items-center gap-1"><span class="w-3 h-3 bg-base-content inline-block"></span> infra</div>
      </div>
    </div>
  </div>
</template>
