<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { Person, CreatePerson, UpdatePerson } from '@/types';

const props = defineProps<{
  person?: Person;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreatePerson | UpdatePerson];
  cancel: [];
}>();

const form = ref<CreatePerson>({
  name: props.initialName || '',
  email: '',
  role: '',
  department: 'GhentCDH',
  phone: '',
  is_active: true,
  notes: '',
});

let emailEdited: boolean = false;

function markEmailEdited() {
  emailEdited = true;
}

watch(
  () => props.person,
  (p) => {
    if (p) {
      form.value = {
        name: p.name,
        email: p.email || '',
        role: p.role || '',
        department: p.department || '',
        phone: p.phone || '',
        is_active: p.is_active,
        notes: p.notes || '',
      };
    }
  },
  { immediate: true }
);

function handleSubmit() {
  emit('submit', form.value);
}

function first_and_last(name: string): { first?: string; last?: string } {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0 || parts[0] === '') return {};
  if (parts.length === 1) return { first: parts[0].toLowerCase() };
  return {
    first: parts[0].toLowerCase(),
    last: parts.slice(1).join('').toLowerCase(),
  };
}

function to_ugent_mail(name: string): string {
  const { first, last } = first_and_last(name);

  return `${first}.${last ?? 'surname'}@ugent.be`;
}

watch(
  () => form.value.name,
  (name) => {
    if (!emailEdited) {
      if (name) {
        form.value.email = to_ugent_mail(name);
      } else {
        form.value.email = '';
      }
    }
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
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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
        <legend class="fieldset-legend">Email</legend>
        <input
          v-model="form.email"
          type="email"
          class="input w-full"
          @input="markEmailEdited"
        />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Role</legend>
        <input v-model="form.role" type="text" class="input w-full" />
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

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Status</legend>
        <label class="flex items-center gap-3 cursor-pointer">
          <input
            v-model="form.is_active"
            type="checkbox"
            class="checkbox checkbox-primary"
          />
          <span>Active</span>
        </label>
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
        {{ person ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
