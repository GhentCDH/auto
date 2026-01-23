<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Client, CreateClient, UpdateClient } from '@/types';

const props = defineProps<{
  client?: Client;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateClient | UpdateClient];
  cancel: [];
}>();

const form = ref<CreateClient>({
  name: props.initialName || '',
  contact_name: '',
  contact_email: '',
  department: '',
  phone: '',
  address: '',
  status: 'active',
  notes: '',
});

watch(
  () => props.client,
  (c) => {
    if (c) {
      form.value = {
        name: c.name,
        contact_name: c.contact_name || '',
        contact_email: c.contact_email || '',
        department: c.department || '',
        phone: c.phone || '',
        address: c.address || '',
        status: c.status,
        notes: c.notes || '',
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
        <legend class="fieldset-legend">Company Name *</legend>
        <input v-model="form.name" type="text" class="input w-full" required />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Status</legend>
        <select v-model="form.status" class="select w-full">
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Contact Name</legend>
        <input v-model="form.contact_name" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Contact Email</legend>
        <input v-model="form.contact_email" type="email" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Department</legend>
        <input v-model="form.department" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Phone</legend>
        <input v-model="form.phone" type="tel" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Address</legend>
        <input v-model="form.address" type="text" class="input w-full" />
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
        {{ client ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
