<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { toast } from 'vue-sonner';
import { useAuth } from '../../composables/useAuth';
import { useConfig } from '../../composables/useConfig';
import { authApi } from '../../api';

const router = useRouter();
const { login } = useAuth();
const { config } = useConfig();

const passwordEnabled = computed(
  () => config.value?.auth.password_enabled ?? true
);
const oidcEnabled = computed(() => config.value?.auth.oidc_enabled ?? false);

const username = ref('');
const password = ref('');
const submitting = ref(false);

async function submit() {
  submitting.value = true;
  try {
    await login(username.value, password.value);
    router.push('/');
  } catch (e: any) {
    toast.error(e.message || 'Login failed');
  } finally {
    submitting.value = false;
  }
}

function loginWithOidc() {
  window.location.assign(authApi.oidcStartUrl);
}

// Forgot-password (logged-out reset request)
const showReset = ref(false);
const resetUsername = ref('');
async function submitReset() {
  try {
    await authApi.resetRequest(resetUsername.value);
    toast.success(
      'If that account exists, an admin has been notified to issue a reset link.'
    );
    showReset.value = false;
    resetUsername.value = '';
  } catch (e: any) {
    toast.error(e.message || 'Could not submit request');
  }
}

// --- Mascot gravity simulation ---------------------------------------------
// Reference tuning: everything below was hand-tuned at this viewport and is
// scaled from it so the field looks the same density at any screen size.
const REF_W = 1236;
const REF_H = 2538;
const REF_COUNT = 20;
const REF_EQUILIBRIUM = 300;
const REF_MARGIN = 100;
const REF_MAX_SPEED = 5;
const REF_EPSILON = 800;

// Sized per-viewport in configure(); template reads the reactive range.
let MASCOT_COUNT = REF_COUNT;
const mascotRange = ref<number[]>([]);

// Plain (non-reactive) physics state — reallocated in configure(), mutated in
// the rAF loop, written straight to DOM transforms so Vue never re-renders.
let x = new Float64Array(MASCOT_COUNT);
let y = new Float64Array(MASCOT_COUNT);
let vx = new Float64Array(MASCOT_COUNT);
let vy = new Float64Array(MASCOT_COUNT);
let mass = new Float64Array(MASCOT_COUNT);

const els: HTMLElement[] = [];
const setEl = (el: any, i: number) => {
  if (el) els[i] = el as HTMLElement;
};

// Bodies live in the container's local coordinate space (it's their offset
// parent), NOT the viewport — `main` is `container mx-auto`, so the two differ.
const rootEl = ref<HTMLElement>();
let bounds = { w: window.innerWidth, h: window.innerHeight };
let rafId = 0;

// Mouse acts as an extra body: it exerts force but is never moved by it.
// Inactive (-1) until the pointer first enters / moves over the page.
let mouseX = -1;
let mouseY = -1;
function onMouseMove(e: MouseEvent) {
  const r = rootEl.value?.getBoundingClientRect();
  if (!r) return;
  mouseX = e.clientX - r.left;
  mouseY = e.clientY - r.top;
}
function onMouseLeave() {
  mouseX = -1;
  mouseY = -1;
}

// Tuned for slow, lazy ambient drift. Dimensionless knobs stay fixed; every
// length/force below is recomputed from the viewport in configure().
const DT = 0.5; // fixed timestep (stability over wall-clock)
const DAMPING = 0.2; // viscous thermostat — cools the lattice instead of letting it vibrate
const SLEEP2 = 1e-2; // (px/step)^2 — below this a body is parked at rest (kills sub-pixel flicker)

let SOFTENING = 1; // px, avoids singularity / blow-up at close range
let SOFTENING2 = SOFTENING * SOFTENING;
let MAX_SPEED = REF_MAX_SPEED; // px/step cap
let MAX_SPEED2 = MAX_SPEED * MAX_SPEED;
let MARGIN = REF_MARGIN; // keep bodies a bit inside the edges
let EQUILIBRIUM = REF_EQUILIBRIUM;
let EPSILON = REF_EPSILON;
let E6 = EQUILIBRIUM ** 6;
let MIN_DIST = EQUILIBRIUM; // min spawn separation

// Derive count + all lengths from the viewport, relative to the reference.
// s = linear scale; equilibrium grows as √s (sub-linear) so count can grow
// as s while the packing fraction (look/density) stays constant.
function configure() {
  const el = rootEl.value;
  bounds = {
    w: el?.clientWidth || window.innerWidth,
    h: el?.clientHeight || window.innerHeight,
  };
  const s = Math.sqrt((bounds.w * bounds.h) / (REF_W * REF_H));
  const eqScale = Math.sqrt(s);

  EQUILIBRIUM = REF_EQUILIBRIUM * eqScale;
  MARGIN = REF_MARGIN * eqScale;
  MAX_SPEED = REF_MAX_SPEED * eqScale;
  SOFTENING = 1 * eqScale;
  // Keep LJ acceleration (~ε/eq²) constant so the motion timescale matches ref.
  EPSILON = REF_EPSILON * (EQUILIBRIUM / REF_EQUILIBRIUM) ** 2;

  E6 = EQUILIBRIUM ** 6;
  MIN_DIST = EQUILIBRIUM;
  SOFTENING2 = SOFTENING * SOFTENING;
  MAX_SPEED2 = MAX_SPEED * MAX_SPEED;

  MASCOT_COUNT = Math.max(1, Math.round(REF_COUNT * s));
  mascotRange.value = Array.from({ length: MASCOT_COUNT }, (_, i) => i);

  x = new Float64Array(MASCOT_COUNT);
  y = new Float64Array(MASCOT_COUNT);
  vx = new Float64Array(MASCOT_COUNT);
  vy = new Float64Array(MASCOT_COUNT);
  mass = new Float64Array(MASCOT_COUNT);
  els.length = 0;
}

function initBodies() {
  for (let i = 0; i < MASCOT_COUNT; i++) {
    let px = 0;
    let py = 0;
    let attempts = 0;
    do {
      px = MARGIN + Math.random() * (bounds.w - MARGIN * 2);
      py = MARGIN + Math.random() * (bounds.h - MARGIN * 2);
      attempts++;
    } while (
      attempts < 1000 &&
      Array.from({ length: i }, (_, j) => {
        const dx = x[j] - px;
        const dy = y[j] - py;
        return dx * dx + dy * dy;
      }).some((d2) => d2 < MIN_DIST * MIN_DIST)
    );

    x[i] = px;
    y[i] = py;
    vx[i] = (Math.random() - 0.5) * 2;
    vy[i] = (Math.random() - 0.5) * 2;
    mass[i] = 1;
  }
}

function render() {
  for (let i = 0; i < MASCOT_COUNT; i++) {
    const el = els[i];
    if (el) {
      el.style.transform = `translate3d(${x[i]}px, ${y[i]}px, 0) translate(-50%, -50%) scale(${mass[i]})`;
    }
  }
}

function step() {
  for (let i = 0; i < MASCOT_COUNT; i++) {
    let ax = 0;
    let ay = 0;
    const xi = x[i];
    const yi = y[i];

    for (let j = 0; j < MASCOT_COUNT; j++) {
      if (i === j) continue;
      const dx = x[j] - xi;
      const dy = y[j] - yi;
      const r2 = dx * dx + dy * dy + SOFTENING2;
      const r6 = r2 ** 3;
      const r12 = r6 ** 2;
      const sig6 = E6;
      const sig12 = sig6 ** 2;

      const lj = ((24 * EPSILON) / r2) * ((2 * sig12) / r12 - sig6 / r6);
      // Force points along (xi - xj) for repulsion; dx points i->j, so subtract.
      ax -= lj * dx;
      ay -= lj * dy;
    }

    // Mouse contributes the same LJ force but is itself immovable.
    if (mouseX >= 0) {
      const dx = mouseX - xi;
      const dy = mouseY - yi;
      const r2 = dx * dx + dy * dy + SOFTENING2;
      const r6 = r2 ** 3;
      const r12 = r6 ** 2;
      const lj = 10.0 * ((24 * EPSILON) / r2) * ((2 * E6 ** 2) / r12 - E6 / r6);
      ax -= lj * dx;
      ay -= lj * dy;
    }

    vx[i] = (vx[i] + ax * DT) * DAMPING;
    vy[i] = (vy[i] + ay * DT) * DAMPING;

    const sp2 = vx[i] * vx[i] + vy[i] * vy[i];
    if (sp2 > MAX_SPEED2) {
      const s = MAX_SPEED / Math.sqrt(sp2);
      vx[i] *= s;
      vy[i] *= s;
    } else if (sp2 < SLEEP2) {
      // Near-zero residual velocity at the lattice → snap to rest, no flicker.
      vx[i] = 0;
      vy[i] = 0;
    }
  }

  for (let i = 0; i < MASCOT_COUNT; i++) {
    x[i] += vx[i] * DT;
    y[i] += vy[i] * DT;

    if (x[i] < MARGIN) {
      x[i] = MARGIN;
      vx[i] = Math.abs(vx[i]);
    } else if (x[i] > bounds.w - MARGIN) {
      x[i] = bounds.w - MARGIN;
      vx[i] = -Math.abs(vx[i]);
    }
    if (y[i] < MARGIN) {
      y[i] = MARGIN;
      vy[i] = Math.abs(vy[i]);
    } else if (y[i] > bounds.h - MARGIN) {
      y[i] = bounds.h - MARGIN;
      vy[i] = -Math.abs(vy[i]);
    }
  }

  render();
  rafId = requestAnimationFrame(step);
}

function onResize() {
  configure();
  initBodies();
}

onMounted(() => {
  configure();
  initBodies();
  render();
  window.addEventListener('resize', onResize);
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseout', onMouseLeave);
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (!reduced) rafId = requestAnimationFrame(step);
});

onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId);
  window.removeEventListener('resize', onResize);
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseout', onMouseLeave);
});
</script>

<template>
  <div
    ref="rootEl"
    class="flex h-screen w-screen items-center flex-col justify-center relative"
  >
    <div
      v-for="i in mascotRange"
      :key="i"
      :ref="(el) => setEl(el, i)"
      class="absolute w-20 h-20 top-0 left-0 will-change-transform pointer-events-none"
    >
      <img src="/favicon.svg" />
    </div>
    <div class="card w-full max-w-sm bg-base-200 shadow-xl md:-mt-100">
      <div class="card-body">
        <h1 class="card-title justify-center text-2xl wallefont">SIGN IN</h1>

        <form
          v-if="passwordEnabled && !showReset"
          @submit.prevent="submit"
          class="space-y-3"
        >
          <input
            v-model="username"
            type="text"
            required
            placeholder="Username"
            autocomplete="username"
            class="input input-bordered w-full"
          />
          <input
            v-model="password"
            type="password"
            required
            placeholder="Password"
            autocomplete="current-password"
            class="input input-bordered w-full"
          />
          <button
            type="submit"
            class="btn btn-primary w-full"
            :disabled="submitting"
          >
            Sign in
          </button>
          <button
            type="button"
            class="btn btn-link btn-sm w-full"
            @click="showReset = true"
          >
            Forgot password?
          </button>
        </form>

        <form
          v-else-if="showReset"
          @submit.prevent="submitReset"
          class="space-y-3"
        >
          <p class="text-sm text-base-content/70">
            Enter your username. An admin will be notified and can issue you a
            reset link.
          </p>
          <input
            v-model="resetUsername"
            type="text"
            required
            placeholder="Username"
            class="input input-bordered w-full"
          />
          <button type="submit" class="btn btn-primary w-full">
            Request reset
          </button>
          <button
            type="button"
            class="btn btn-link btn-sm w-full"
            @click="showReset = false"
          >
            Back to sign in
          </button>
        </form>

        <div v-if="oidcEnabled && !showReset">
          <div v-if="passwordEnabled" class="divider text-xs">OR</div>
          <button class="btn btn-outline w-full" @click="loginWithOidc">
            Sign in with SSO
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
