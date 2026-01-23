<script setup lang="ts">
defineProps<{
  title: string;
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': open }" :open="open">
    <div class="modal-box">
      <h3 class="text-lg font-bold">{{ title }}</h3>
      <button
        class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
        @click="emit('close')"
      >
        X
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
</template>
