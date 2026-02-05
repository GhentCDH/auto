<script setup lang="ts">
import { watch, onUnmounted } from 'vue';
import { X } from 'lucide-vue-next';

const props = defineProps<{
  title: string;
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close');
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      document.addEventListener('keydown', handleKeydown);
    } else {
      document.removeEventListener('keydown', handleKeydown);
    }
  }
);

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <dialog class="modal" :class="{ 'modal-open': open }">
      <div v-if="open" class="modal-box">
        <h3 class="text-lg font-bold">{{ title }}</h3>
        <button
          class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
          @click="emit('close')"
        >
          <X />
        </button>
        <div class="py-4">
          <slot />
        </div>
        <div class="modal-action">
          <slot name="actions" />
        </div>
      </div>
      <form method="dialog" class="modal-backdrop">
        <button @click="emit('close')">close</button>
      </form>
    </dialog>
  </Teleport>
</template>
