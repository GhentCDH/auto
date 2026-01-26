<script setup lang="ts">
import { computed, ref } from 'vue';
import type { LinkHost, HostRelation } from '@/types';
import { hostRoles } from '@/values';

const props = defineProps<{
  hostName: string;
  initial?: HostRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkHost];
  cancel: [];
}>();

// Check if initial role is in hostRoles, if not it's a custom role
const initialIsCustom = props.initial?.role && !Object.keys(hostRoles).includes(props.initial.role);
const selectedRole = ref(initialIsCustom ? 'other' : (props.initial?.role || 'production'));
const customRole = ref(initialIsCustom ? props.initial?.role || '' : '');
const notes = ref(props.initial?.relation_notes || '');

const form = computed<LinkHost>(() => ({
  role: selectedRole.value === 'other' ? customRole.value : selectedRole.value,
  notes: notes.value,
}));

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
      <select v-model="selectedRole" class="select w-full">
        <option v-for="(visual, value) in hostRoles" :value="value">
          {{ visual }}
        </option>
      </select>
      <input
        v-if="selectedRole === 'other'"
        v-model="customRole"
        type="text"
        class="input w-full mt-2"
        placeholder="Enter custom role"
        required
      />
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
      <button type="submit" class="btn btn-primary">
        {{ initial ? 'Update' : 'Link Host' }}
      </button>
    </div>
  </form>
</template>
