<script setup lang="ts">
import { ref } from 'vue';
import type { LinkPerson, PersonRelation } from '@/types';
import { contributionTypes } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  personName: string;
  initial?: PersonRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkPerson];
  cancel: [];
}>();

const form = ref({
  contribution_type: props.initial?.contribution_type || 'developer',
  start_date: props.initial?.start_date || '',
  end_date: props.initial?.end_date || '',
  notes: props.initial?.relation_notes || '',
} satisfies LinkPerson);

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ personName }}</span> to this
      application
    </p>

    <SelectWithCustom
      v-model="form.contribution_type"
      :options="contributionTypes"
      label="Contribution Type"
      allow-custom
      custom-placeholder="Enter custom contribution"
    />

    <div class="grid grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Start Date</legend>
        <input v-model="form.start_date" type="date" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">End Date</legend>
        <input v-model="form.end_date" type="date" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea
        v-model="form.notes"
        class="textarea w-full"
        rows="2"
        placeholder="Optional notes"
      />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">
        {{ initial ? 'Update' : 'Link Person' }}
      </button>
    </div>
  </form>
</template>
