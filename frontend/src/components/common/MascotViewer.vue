<script setup lang="ts">
import { computed } from 'vue';
import { useMouseLookAt } from '@/composables/useMouseLookAt';

interface Props {
  size?: number;
  proximityRadius?: number;
  depth?: number;
  layers?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 40,
  proximityRadius: 150,
  depth: 5,
  layers: 5,
});

const { rotationX, rotationY, containerRef } = useMouseLookAt({
  proximityRadius: props.proximityRadius,
  maxRotationX: 0.7,
  maxRotationY: 0.7,
});

const sizeStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
}));

const rotX = computed(() => -rotationX.value * (180 / Math.PI));
const rotY = computed(() => rotationY.value * (180 / Math.PI));

const baseBrightness = computed(() => {
  // Light source from upper-right: darker when looking down, lighter when looking right
  const downEffect = -rotationX.value * 0.4;
  const rightEffect = rotationY.value * 0.15;
  return 1 + rightEffect + downEffect;
});

function getLayerStyle(index: number) {
  const zOffset = (index / (props.layers - 1)) * props.depth;
  const layerDarkness = (1 - index / props.layers) * 0.6; // back layers are darker
  const brightness = Math.max(0, baseBrightness.value - layerDarkness);

  return {
    transform: `perspective(200px) rotateX(${rotX.value}deg) rotateY(${rotY.value}deg) translateZ(${zOffset}px)`,
    filter: `brightness(${brightness})`,
    zIndex: index,
  };
}
</script>

<template>
  <div ref="containerRef" class="mascot-viewer" :style="sizeStyle">
    <div class="layers-container">
      <img
        v-for="i in layers"
        :key="i"
        src="/auto.webp"
        :alt="i === layers ? 'Auto mascot' : ''"
        class="mascot-layer"
        :style="getLayerStyle(i - 1)"
      />
    </div>
  </div>
</template>

<style scoped>
.mascot-viewer {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.layers-container {
  position: relative;
  width: 100%;
  height: 100%;
  transform-style: preserve-3d;
}

.mascot-layer {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  transition:
    transform 0.05s ease-out,
    filter 0.05s ease-out;
}
</style>
