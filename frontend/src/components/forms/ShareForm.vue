<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type {
  NetworkShare,
  CreateNetworkShare,
  UpdateNetworkShare,
} from '@/types';
import { shareTypes } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  share?: NetworkShare;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateNetworkShare | UpdateNetworkShare];
  cancel: [];
}>();

let pathChanged: boolean = false;

function markPathChanged() {
  pathChanged = true;
}

const form = ref({
  name: props.initialName || '',
  path: '/ghentcdh_',
  share_type: 'smb',
  server: 'files.ugent.be',
  purpose: '',
  status: 'active',
  notes: '',
} satisfies CreateNetworkShare);

watch(
  () => props.share,
  (s) => {
    if (s) {
      form.value = {
        name: s.name,
        path: s.path,
        share_type: s.share_type,
        server: s.server || '',
        purpose: s.purpose || '',
        status: s.status,
        notes: s.notes || '',
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
    if (!pathChanged) {
      form.value.path = `/ghentcdh_${name}`;
    }
  },
  { immediate: true }
);

// if path changed manually, stop following the name in the watch, by setting pathChanged to true

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

      <SelectWithCustom
        v-model="form.share_type"
        :options="shareTypes"
        label="Type"
        allow-custom
      />

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Path *</legend>
        <input
          v-model="form.path"
          type="text"
          class="input w-full"
          placeholder="//server/share or /mnt/share"
          required
          @input="markPathChanged"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Server</legend>
        <input v-model="form.server" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Purpose</legend>
        <input v-model="form.purpose" type="text" class="input w-full" />
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Status</legend>
        <select v-model="form.status" class="select w-full">
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
          <option value="deprecated">Deprecated</option>
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
        {{ share ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
