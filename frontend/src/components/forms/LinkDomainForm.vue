<script setup lang="ts">
import { ref } from 'vue';
import type { LinkDomain, DomainRelation } from '@/types';

const props = defineProps<{
  domainName: string;
  initial?: DomainRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkDomain];
  cancel: [];
}>();

const form = ref({
  notes: props.initial?.relation_notes || '',
} satisfies LinkDomain);

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
        {{ initial ? 'Update' : 'Link Domain' }}
      </button>
    </div>
  </form>
</template>
