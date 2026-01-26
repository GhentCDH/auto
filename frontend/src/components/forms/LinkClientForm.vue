<script setup lang="ts">
import { ref } from 'vue';
import type { LinkClient, ClientRelation } from '@/types';
import { relationshipTypes } from '@/values';

const props = defineProps<{
  clientName: string;
  initial?: ClientRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkClient];
  cancel: [];
}>();

const form = ref<LinkClient>({
  relationship_type: props.initial?.relationship_type || 'customer',
  contract_ref: props.initial?.contract_ref || '',
  notes: props.initial?.relation_notes || '',
});

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ clientName }}</span> to this
      application
    </p>

    <div class="grid grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Relationship Type</legend>
        <select v-model="form.relationship_type" class="select w-full">
          <option v-for="(visual, value) in relationshipTypes" :value="value">
            {{ visual }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Contract Reference</legend>
        <input
          v-model="form.contract_ref"
          type="text"
          class="input w-full"
          placeholder="Contract #"
        />
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
        {{ initial ? 'Update' : 'Link Client' }}
      </button>
    </div>
  </form>
</template>
