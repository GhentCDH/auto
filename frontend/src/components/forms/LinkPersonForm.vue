<script setup lang="ts">
import { ref } from 'vue';
import type { LinkPerson } from '@/types';
import { contributionTypes } from '@/values';

defineProps<{
  personName: string;
}>();

const emit = defineEmits<{
  submit: [data: LinkPerson];
  cancel: [];
}>();

const form = ref<LinkPerson>({
  contribution_type: 'developer',
  start_date: '',
  end_date: '',
  notes: '',
});

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

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Contribution Type</legend>
      <select v-model="form.contribution_type" class="select w-full">
        <option v-for="(visual, value) in contributionTypes" :value="value">
          {{ visual }}
        </option>
      </select>
    </fieldset>

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
      <button type="submit" class="btn btn-primary">Link Person</button>
    </div>
  </form>
</template>
