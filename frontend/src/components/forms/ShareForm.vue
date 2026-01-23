<script setup lang="ts">
import { ref, watch } from 'vue';
import type {
  NetworkShare,
  CreateNetworkShare,
  UpdateNetworkShare,
} from '@/types';

const props = defineProps<{
  share?: NetworkShare;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateNetworkShare | UpdateNetworkShare];
  cancel: [];
}>();

const form = ref<CreateNetworkShare>({
  name: props.initialName || '',
  path: '',
  share_type: 'smb',
  server: '',
  purpose: '',
  status: 'active',
  notes: '',
});

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
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Name *</legend>
        <input v-model="form.name" type="text" class="input w-full" required />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Type</legend>
        <select v-model="form.share_type" class="select w-full">
          <option value="smb">SMB</option>
          <option value="nfs">NFS</option>
          <option value="cifs">CIFS</option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Path *</legend>
        <input
          v-model="form.path"
          type="text"
          class="input w-full"
          placeholder="//server/share or /mnt/share"
          required
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
