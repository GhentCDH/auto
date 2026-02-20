<script setup lang="ts">
import { Copy, Package } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const props = defineProps<{
  imageRef: string;
  alias?: string;
}>();

const imageUrl = computed(() => {
  const ref_ = props.imageRef;
  if (ref_.startsWith('http://') || ref_.startsWith('https://')) return ref_;
  const host = ref_.includes('.') ? ref_.split('/')[0] : 'docker.io';
  return `https://${ref_}`;
});

const copied = ref(false);

function copy() {
  navigator.clipboard.writeText(props.imageRef);
  copied.value = true;
  setTimeout(() => (copied.value = false), 600);
}
</script>

<template>
  <a
    :href="imageUrl"
    target="_blank"
    class="badge badge-outline gap-1 hover:badge-primary"
  >
    <Package class="w-3 h-3" />
    {{ alias || imageRef }}
  </a>
  <button
    @click="copy"
    class="p-0 relative cursor-pointer group"
    :class="{ copied }"
  >
    <Copy class="w-3 h-3 transition-all group-hover:stroke-3" />
    <span v-if="copied" class="sparkles" aria-hidden="true">
      <span v-for="i in 6" :key="i" class="spark" :style="`--i: ${i}`" />
    </span>
  </button>
</template>

<style scoped>
button.copied {
  animation: pop 0.3s ease;
}

@keyframes pop {
  0% {
    transform: scale(1);
  }
  40% {
    transform: scale(0.75) rotate(-10deg);
  }
  70% {
    transform: scale(1.2) rotate(5deg);
  }
  100% {
    transform: scale(1);
  }
}

.sparkles {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.spark {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: oklch(70% 0.2 calc(var(--i) * 60));
  animation: spark 0.5s ease forwards;
  transform-origin: center;
  --angle: calc(var(--i) * 60deg);
}

@keyframes spark {
  0% {
    transform: translate(-50%, -50%) rotate(var(--angle)) translateY(0) scale(1);
    opacity: 1;
  }
  100% {
    transform: translate(-50%, -50%) rotate(var(--angle)) translateY(-24px)
      scale(0);
    opacity: 0;
  }
}
</style>
