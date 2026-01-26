<script setup lang="ts">
import { computed, ref } from 'vue';
import type { LinkDomain, DomainRelation } from '@/types';
import { domainTypes } from '@/values';
import EntitySelector from '../common/EntitySelector.vue';
import { hostsApi } from '@/api';
import { Check, X } from 'lucide-vue-next';

const props = defineProps<{
  domainName: string;
  initial?: DomainRelation;
}>();

const emit = defineEmits<{
  submit: [data: LinkDomain];
  cancel: [];
}>();

type TargetType = 'host' | 'free';

// Determine initial target type based on existing data
const initialTargetType: TargetType = props.initial?.target_host_id ? 'host' : (props.initial?.target ? 'free' : 'host');

const target_type = ref<TargetType>(initialTargetType);
const record_type = ref(props.initial?.record_type || 'A');
const target = ref(props.initial?.target || '');
const target_host_id = ref(props.initial?.target_host_id || '');
const target_host_name = ref<null | string>(props.initial?.target_host_name || null);
const is_primary = ref(props.initial?.is_primary || false);
const notes = ref(props.initial?.relation_notes || '');

const form = computed<LinkDomain>(() => ({
  record_type: record_type.value,
  target: target_type.value == 'free' ? target.value : null,
  target_host_id: target_type.value == 'host' ? target_host_id.value : null,
  is_primary: is_primary.value,
  notes: notes.value,
}));

function handleHostSelect(entity: { id: string; name: string }) {
  target_host_id.value = entity.id;
  target_host_name.value = entity.name;
}

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

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Record Type</legend>
      <select v-model="record_type" class="select w-full">
        <option v-for="(visual, value) in domainTypes" :value="value">
          {{ visual }}
        </option>
      </select>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Target Type</legend>
      <select v-model="target_type" class="select w-full">
        <option value="host">Host</option>
        <option value="free">Free Entry</option>
      </select>
      <input
        v-if="target_type == 'free'"
        v-model="target"
        type="text"
        class="input w-full"
        placeholder="IP or hostname"
      />
      <EntitySelector
        v-else-if="target_host_name === null"
        title="Target Host"
        :fetchFn="hostsApi.list"
        @select="handleHostSelect"
      />
    </fieldset>

    <fieldset class="fieldset" v-if="target_host_name !== null">
      <legend class="fieldset-legend">Selected Target Host</legend>
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <Check class="h-4 w-4 text-success" />
          {{ target_host_name }}
        </div>
        <X
          class="h-4 w-4 text-error cursor-pointer"
          @click="target_host_name = null"
        />
      </div>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Primary Domain</legend>
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          v-model="is_primary"
          type="checkbox"
          class="checkbox checkbox-primary"
        />
        <span>This is the primary domain for this application</span>
      </label>
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea
        v-model="notes"
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
