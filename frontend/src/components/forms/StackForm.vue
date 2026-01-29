<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { Stack, CreateStack, UpdateStack } from '@/types';

const props = defineProps<{
  stack?: Stack;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateStack | UpdateStack];
  cancel: [];
}>();

const form = ref<CreateStack>({
  name: props.initialName || '',
  notes: '',
});

watch(
  () => props.stack,
  (s) => {
    if (s) {
      form.value = {
        name: s.name,
        notes: s.notes || '',
      };
    }
  },
  { immediate: true }
);

function handleSubmit() {
  emit('submit', form.value);
}

const nameInput = ref<HTMLInputElement>();

onMounted(() => {
  nameInput.value?.focus();
});
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Name *</legend>
      <input
        v-model="form.name"
        ref="nameInput"
        type="text"
        class="input w-full"
        placeholder="e.g., TypeScript, React, PostgreSQL"
        required
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea
        v-model="form.notes"
        class="textarea w-full"
        rows="3"
        placeholder="Optional notes about this technology"
      />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ stack ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
