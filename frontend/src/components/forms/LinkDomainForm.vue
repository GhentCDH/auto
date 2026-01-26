<script setup lang="ts">
import { ref } from 'vue';
import type { LinkDomain } from '@/types';
import { domainTypes } from '@/values';

defineProps<{
  domainName: string;
}>();

const emit = defineEmits<{
  submit: [data: LinkDomain];
  cancel: [];
}>();

const form = ref<LinkDomain>({
  record_type: 'A',
  target: '',
  is_primary: false,
  notes: '',
});

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ domainName }}</span> to this
      application
    </p>

    <div class="grid grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Record Type</legend>
        <select v-model="form.record_type" class="select w-full">
          <option v-for="(visual, value) in domainTypes" :value="value">
            {{ visual }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Target</legend>
        <input
          v-model="form.target"
          type="text"
          class="input w-full"
          placeholder="IP or hostname"
        />
        <div class="label">optional</div>
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Primary Domain</legend>
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          v-model="form.is_primary"
          type="checkbox"
          class="checkbox checkbox-primary"
        />
        <span>This is the primary domain for this application</span>
      </label>
    </fieldset>

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
      <button type="submit" class="btn btn-primary">Link Domain</button>
    </div>
  </form>
</template>
