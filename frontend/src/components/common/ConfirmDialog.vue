<script setup lang="ts">
defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': open }">
    <div class="modal-box">
      <h3 class="text-lg font-bold">{{ title }}</h3>
      <p class="py-4">{{ message }}</p>
      <div class="modal-action">
        <button class="btn" @click="emit('cancel')">Cancel</button>
        <button
          class="btn"
          :class="danger ? 'btn-error' : 'btn-primary'"
          @click="emit('confirm')"
        >
          {{ confirmLabel || 'Confirm' }}
        </button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop">
      <button @click="emit('cancel')">close</button>
    </form>
  </dialog>
</template>
