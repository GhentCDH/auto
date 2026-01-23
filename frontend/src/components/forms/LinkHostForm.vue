<script setup lang="ts">
import { ref } from 'vue';
import type { LinkHost } from '@/types';

defineProps<{
  hostName: string;
}>();

const emit = defineEmits<{
  submit: [data: LinkHost];
  cancel: [];
}>();

const form = ref<LinkHost>({
  role: 'primary',
  notes: '',
});

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ hostName }}</span> to this application
    </p>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Role</legend>
      <select v-model="form.role" class="select w-full">
        <option value="primary">Primary</option>
        <option value="backup">Backup</option>
        <option value="staging">Staging</option>
        <option value="development">Development</option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea
        v-model="form.notes"
        class="textarea w-full"
        rows="2"
        placeholder="Optional notes about this relationship"
      />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">Link Host</button>
    </div>
  </form>
</template>
