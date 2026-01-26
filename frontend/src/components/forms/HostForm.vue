<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Host, CreateHost, UpdateHost } from '@/types';
import { hostTypes } from '@/values';

const props = defineProps<{
  host?: Host;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateHost | UpdateHost];
  cancel: [];
}>();

const form = ref<CreateHost>({
  name: props.initialName || '',
  host_type: 'physical',
  hostname: '',
  ip_address: '',
  location: '',
  os: '',
  specs: '',
  status: 'active',
  notes: '',
});

watch(
  () => props.host,
  (h) => {
    if (h) {
      form.value = {
        name: h.name,
        host_type: h.host_type,
        hostname: h.hostname || '',
        ip_address: h.ip_address || '',
        location: h.location || '',
        os: h.os || '',
        specs: h.specs || '',
        status: h.status,
        notes: h.notes || '',
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
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Name *</legend>
        <input v-model="form.name" type="text" class="input w-full" required />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Type *</legend>
        <select v-model="form.host_type" class="select w-full" required>
          <option v-for="(visual, value) in hostTypes" :value="value">
            {{ visual }}
          </option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Hostname</legend>
        <input v-model="form.hostname" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">IP Address</legend>
        <input v-model="form.ip_address" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Location</legend>
        <input v-model="form.location" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Operating System</legend>
        <input v-model="form.os" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Specs</legend>
        <input v-model="form.specs" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Status</legend>
        <select v-model="form.status" class="select w-full">
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
          <option value="maintenance">Maintenance</option>
          <option value="decommissioned">Decommissioned</option>
        </select>
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea v-model="form.notes" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ host ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
