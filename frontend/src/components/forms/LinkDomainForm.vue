<script setup lang="ts">
import { ref } from 'vue';
import type { LinkDomain, DomainRelation } from '@/types';
import { domainTypes } from '@/values';
import EntityOrFreeInput from '../common/EntityOrFreeInput.vue';
import { hostsApi } from '@/api';
import SelectWithCustom from '../common/SelectWithCustom.vue';

const props = defineProps<{
  domainName: string;
  initial?: DomainRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkDomain];
  cancel: [];
}>();

const form = ref({
  record_type: props.initial?.record_type || 'A',
  target: props.initial?.target || null,
  target_host_id: props.initial?.target_host_id || null,
  is_primary: props.initial?.is_primary || false,
  notes: props.initial?.relation_notes || '',
} satisfies LinkDomain);

function handleSubmit() {
  emit('submit', form.value);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <p class="text-sm text-base-content/70">
      Link <span class="font-semibold">{{ domainName }}</span> to this
      application
    </p>

    <SelectWithCustom
      v-model="form.record_type"
      :options="domainTypes"
      label="Record Type"
      allow-custom
    />

    <EntityOrFreeInput
      label="Target"
      entity-label="Host"
      free-label="Free Entry"
      free-placeholder="IP or hostname"
      entity-title="Hosts"
      :fetch-fn="hostsApi.list"
      :initial-entity-id="initial?.target_host_id"
      :initial-entity-name="initial?.target_host_name"
      :initial-free-value="initial?.target"
      @update:entity-id="form.target_host_id = $event"
      @update:free-value="form.target = $event"
    />

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Primary Domain</legend>
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          v-model="form.is_primary"
          type="checkbox"
          class="checkbox checkbox-primary"
        />
        <span>This is the primary domain for this application</span>
      </label>
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
        {{ initial ? 'Update' : 'Link Domain' }}
      </button>
    </div>
  </form>
</template>
