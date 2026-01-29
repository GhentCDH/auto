<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type {
  Application,
  CreateApplication,
  UpdateApplication,
} from '@/types';
import { environments } from '@/values';

const props = defineProps<{
  application?: Application;
}>();

const emit = defineEmits<{
  submit: [data: CreateApplication | UpdateApplication];
  cancel: [];
}>();

const form = ref<CreateApplication>({
  name: '',
  description: '',
  repository_url: '',
  environment: 'prd',
  url: '',
  status: 'active',
});

watch(
  () => props.application,
  (app) => {
    if (app) {
      form.value = {
        name: app.name,
        description: app.description || '',
        repository_url: app.repository_url || '',
        environment: app.environment,
        url: app.url || '',
        status: app.status,
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
      <legend class="fieldset-legend">URL</legend>
      <input
        v-model="form.url"
        type="url"
        class="input w-full"
        placeholder="https://..."
      />
      <div class="label">optional - main URL to access the application</div>
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
        {{ application ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
