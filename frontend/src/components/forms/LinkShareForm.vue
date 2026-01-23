<script setup lang="ts">
import { ref } from 'vue';
import type { LinkNetworkShare } from '@/types';

defineProps<{
  shareName: string;
}>();

const emit = defineEmits<{
  submit: [data: LinkNetworkShare];
  cancel: [];
}>();

const form = ref<LinkNetworkShare>({
  usage: '',
  mount_point: '',
  permissions: 'read-write',
  notes: '',
});

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ shareName }}</span> to this
      application
    </p>

    <div class="grid grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Usage</legend>
        <select v-model="form.usage" class="select w-full">
          <option value="">Select usage...</option>
          <option value="config">Configuration</option>
          <option value="data">Data Storage</option>
          <option value="logs">Logs</option>
          <option value="backup">Backup</option>
          <option value="media">Media</option>
        </select>
        <div class="label">optional</div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Permissions</legend>
        <select v-model="form.permissions" class="select w-full">
          <option value="read">Read Only</option>
          <option value="write">Write Only</option>
          <option value="read-write">Read/Write</option>
        </select>
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Mount Point</legend>
      <input
        v-model="form.mount_point"
        type="text"
        class="input w-full"
        placeholder="/mnt/share or D:\Share"
      />
      <div class="label">optional</div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea
        v-model="form.notes"
        class="textarea w-full"
        rows="2"
        placeholder="Optional notes"
      />
      <div class="label">optional</div>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">Link Share</button>
    </div>
  </form>
</template>
