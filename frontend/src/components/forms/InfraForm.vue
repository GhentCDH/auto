<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Infra, CreateInfra, UpdateInfra } from '@/types';
import { infraTypes } from '@/values';

const props = defineProps<{
  infra?: Infra;
}>();

const emit = defineEmits<{
  submit: [data: CreateInfra | UpdateInfra];
  cancel: [];
}>();

const form = ref<CreateInfra>({
  name: '',
  description: '',
  type: 'server',
});

watch(
  () => props.infra,
  (inf) => {
    if (inf) {
      form.value = {
        name: inf.name,
        description: inf.description || '',
        type: inf.type,
      };
    }
  },
  { immediate: true }
);

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Name *</legend>
      <input v-model="form.name" type="text" class="input w-full" required />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Description</legend>
      <textarea v-model="form.description" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Type *</legend>
      <select v-model="form.type" class="select w-full" required>
        <option v-for="(label, value) in infraTypes" :key="value" :value="value">
          {{ label }}
        </option>
      </select>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ infra ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
