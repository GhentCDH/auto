<script setup lang="ts">
import { ref } from 'vue';
import type { LinkHost, HostRelation } from '@/types';
import { hostRoles } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  hostName: string;
  initial?: HostRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkHost];
  cancel: [];
}>();

// Check if initial role is in hostRoles, if not it's a custom role
const form = ref({
  role: props.initial?.role || 'production',
  notes: props.initial?.relation_notes || '',
} satisfies LinkHost);

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ hostName }}</span> to this application
    </p>

    <SelectWithCustom
      v-model="form.role"
      :options="hostRoles"
      label="Role"
      allow-custom
      custom-placeholder="Enter custom role"
    />

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
      <button type="submit" class="btn btn-primary">
        {{ initial ? 'Update' : 'Link Host' }}
      </button>
    </div>
  </form>
</template>
