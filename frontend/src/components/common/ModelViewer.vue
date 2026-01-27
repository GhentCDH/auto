<script setup lang="ts">
import { TresCanvas } from '@tresjs/core';
import { useGLTF } from '@tresjs/cientos';
import { ref, watch, computed } from 'vue';
import { useMouseLookAt } from '@/composables/useMouseLookAt';

interface Props {
  modelPath: string;
  size?: number;
  proximityRadius?: number;
  cameraDistance?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 40,
  proximityRadius: 1000,
  cameraDistance: 8,
});

const { rotationX, rotationY, containerRef } = useMouseLookAt({
  proximityRadius: props.proximityRadius,
});

const isLoading = ref(true);
const hasError = ref(false);

const { state: gltf, isLoading: gltfLoading } = await useGLTF(props.modelPath, {
  draco: true,
});

const scene = computed(() => gltf.value?.scene ?? null);

watch(gltfLoading, (loading) => {
  isLoading.value = loading;
});

isLoading.value = false;

const sizeStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}));
</script>

<template>
  <div ref="containerRef" class="model-viewer" :style="sizeStyle">
    <div v-if="isLoading" class="loading-spinner">
      <span class="loading loading-spinner loading-xs"></span>
    </div>
    <div v-else-if="hasError" class="error-state">!</div>
    <TresCanvas
      v-else
      clear-color="#00000000"
      render-mode="on-demand"
      :antialias="true"
      class="tres-canvas"
    >
      <TresPerspectiveCamera :position="[0, 0, cameraDistance]" />
      <TresAmbientLight :intensity="0.6" />
      <TresDirectionalLight :position="[5, 5, 5]" :intensity="1" />
      <TresGroup :rotation-x="rotationX" :rotation-y="rotationY">
        <primitive v-if="scene" :object="scene" />
      </TresGroup>
    </TresCanvas>
  </div>
</template>

<style scoped>
.model-viewer {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.tres-canvas {
  width: 100% !important;
  height: 100% !important;
}

.loading-spinner,
.error-state {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.error-state {
  color: var(--fallback-er, oklch(var(--er)));
  font-weight: bold;
}
</style>
