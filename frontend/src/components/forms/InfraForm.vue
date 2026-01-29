<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import type { Infra, CreateInfra, UpdateInfra } from '@/types';
import { infraTypes } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  infra?: Infra;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateInfra | UpdateInfra];
  cancel: [];
}>();

const form = ref<CreateInfra>({
  name: props.initialName || '',
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

watch(
  () => form.value.name,
  (name) => {
    if (name.startsWith('gcdh')) form.value.type = 'vm';
  },
  { immediate: true }
);

const nameInput = ref<HTMLInputElement>();

onMounted(() => {
  nameInput.value?.focus();
});
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Name *</legend>
      <input
        v-model="form.name"
        type="text"
        ref="nameInput"
        class="input w-full"
        required
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Description</legend>
      <textarea v-model="form.description" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <SelectWithCustom v-model="form.type" :options="infraTypes" allow-custom />

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ infra ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
