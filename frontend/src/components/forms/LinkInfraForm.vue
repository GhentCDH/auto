<script setup lang="ts">
import { ref } from 'vue';
import type { LinkInfra } from '@/types';

const emit = defineEmits<{
  submit: [data: LinkInfra];
  cancel: [];
}>();

const form = ref<LinkInfra>({
  notes: '',
});

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea v-model="form.notes" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">Link</button>
    </div>
  </form>
</template>
