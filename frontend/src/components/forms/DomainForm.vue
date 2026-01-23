<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Domain, CreateDomain, UpdateDomain } from '@/types';

const props = defineProps<{
  domain?: Domain;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateDomain | UpdateDomain];
  cancel: [];
}>();

const form = ref<CreateDomain>({
  name: props.initialName || '',
  registrar: '',
  dns_provider: '',
  expires_at: '',
  ssl_expires_at: '',
  ssl_issuer: '',
  status: 'active',
  notes: '',
});

watch(
  () => props.domain,
  (d) => {
    if (d) {
      form.value = {
        name: d.name,
        registrar: d.registrar || '',
        dns_provider: d.dns_provider || '',
        expires_at: d.expires_at || '',
        ssl_expires_at: d.ssl_expires_at || '',
        ssl_issuer: d.ssl_issuer || '',
        status: d.status,
        notes: d.notes || '',
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
        <legend class="fieldset-legend">Domain Name *</legend>
        <input
          v-model="form.name"
          type="text"
          class="input w-full"
          placeholder="example.com"
          required
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Status</legend>
        <select v-model="form.status" class="select w-full">
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
          <option value="expired">Expired</option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Registrar</legend>
        <input v-model="form.registrar" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">DNS Provider</legend>
        <input v-model="form.dns_provider" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Domain Expires</legend>
        <input v-model="form.expires_at" type="date" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">SSL Expires</legend>
        <input v-model="form.ssl_expires_at" type="date" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">SSL Issuer</legend>
        <input v-model="form.ssl_issuer" type="text" class="input w-full" />
        <div class="label">optional</div>
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
        {{ domain ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
