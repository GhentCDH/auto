<script setup lang="ts">
import { ref } from 'vue';
import type { LinkNetworkShare, NetworkShareRelation } from '@/types';
import { shareUsages } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  shareName: string;
  initial?: NetworkShareRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkNetworkShare];
  cancel: [];
}>();

const form = ref({
  usage: props.initial?.usage || 'data',
  mount_point: props.initial?.mount_point || '',
  permissions: props.initial?.permissions || 'read-write',
  notes: props.initial?.relation_notes || '',
} satisfies LinkNetworkShare);

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
      <SelectWithCustom
        v-model="form.usage"
        label="Usage"
        :options="shareUsages"
        allow-custom
      />

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
      <button type="submit" class="btn btn-primary">
        {{ initial ? 'Update' : 'Link Storage' }}
      </button>
    </div>
  </form>
</template>
