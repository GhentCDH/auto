<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { Service, CreateService, UpdateService } from '@/types';
import { environments } from '@/values';

const props = defineProps<{
  service?: Service;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateService | UpdateService];
  cancel: [];
}>();

const form = ref<CreateService>({
  name: props.initialName || '',
  description: '',
  repository_url: '',
  environment: 'prd',
  status: 'active',
});

watch(
  () => props.service,
  (svc) => {
    if (svc) {
      form.value = {
        name: svc.name,
        description: svc.description || '',
        repository_url: svc.repository_url || '',
        environment: svc.environment,
        status: svc.status,
      };
    }
  },
  { immediate: true }
);

function handleSubmit() {
  emit('submit', form.value);
}

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
        ref="nameInput"
        type="text"
        class="input w-full"
        required
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Description</legend>
      <textarea v-model="form.description" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Repository URL</legend>
      <input
        v-model="form.repository_url"
        type="url"
        class="input w-full"
        placeholder="https://github.com/..."
      />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Environment</legend>
      <select v-model="form.environment" class="select w-full">
        <option
          v-for="(label, value) in environments"
          :key="value"
          :value="value"
        >
          {{ label }}
        </option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Status</legend>
      <select v-model="form.status" class="select w-full">
        <option value="active">Active</option>
        <option value="inactive">Inactive</option>
        <option value="deprecated">Deprecated</option>
        <option value="archived">Archived</option>
      </select>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ service ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
