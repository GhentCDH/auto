<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import type { Domain, CreateDomain, UpdateDomain } from '@/types';
import EntitySelector from '../common/EntitySelector.vue';
import { applicationsApi, servicesApi } from '@/api';

const target_type = ref<'application' | 'service'>('application');
const showTargetSelector = ref(false);
const selectedName = ref<string | null>(null);

const props = defineProps<{
  domain?: Domain;
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateDomain | UpdateDomain];
  cancel: [];
}>();

const form = ref<CreateDomain>({
  fqdn: props.initialName || '',
  registrar: '',
  dns_provider: '',
  expires_at: '',
  notes: '',
});

watch(
  () => props.domain,
  (d) => {
    if (d) {
      form.value = {
        fqdn: d.fqdn,
        registrar: d.registrar || '',
        dns_provider: d.dns_provider || '',
        expires_at: d.expires_at || '',
        target_application_id: d.target_application_id || undefined,
        target_service_id: d.target_service_id || undefined,
        notes: d.notes || '',
      };
      target_type.value = d.target_application_id ? 'application' : 'service';
      selectedName.value =
        d.target_application_name || d.target_service_name || null;
    }
  },
  { immediate: true }
);

function validate() {
  return (
    (form.value.target_application_id === undefined &&
      form.value.target_service_id !== undefined) ||
    (form.value.target_application_id !== undefined &&
      form.value.target_service_id === undefined)
  );
}

function handleSubmit() {
  if (validate()) {
    emit('submit', form.value);
  }
  // TODO: handle bad validates! (disable button!)
}

const nameInput = ref<HTMLInputElement>();

onMounted(() => {
  nameInput.value?.focus();
});

function handleApplicationSelect(application: { id: string; name: string }) {
  form.value.target_service_id = undefined;
  form.value.target_application_id = application.id;
  showTargetSelector.value = false;
  selectedName.value = application.name;
}

function handleServiceSelect(service: { id: string; name: string }) {
  form.value.target_service_id = service.id;
  form.value.target_application_id = undefined;
  showTargetSelector.value = false;
  selectedName.value = service.name;
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <fieldset class="fieldset">
        <legend class="fieldset-legend">Domain Name *</legend>
        <input
          v-model="form.fqdn"
          ref="nameInput"
          type="text"
          class="input w-full"
          placeholder="example.com"
          required
          autofocus
        />
      </fieldset>

      <fieldset class="fieldset col-span-2">
        <legend class="fieldset-legend">Target *</legend>
        <div class="flex gap-4 mb-2">
          <label class="label cursor-pointer gap-2">
            <input
              type="radio"
              name="target_type"
              value="application"
              v-model="target_type"
              class="radio radio-primary"
              @change="
                showTargetSelector = true;
                form.target_service_id = undefined;
              "
            />
            <span>Application</span>
          </label>
          <label class="label cursor-pointer gap-2">
            <input
              type="radio"
              name="target_type"
              value="service"
              v-model="target_type"
              class="radio radio-primary"
              @change="
                showTargetSelector = true;
                form.target_application_id = undefined;
              "
            />
            <span>Service</span>
          </label>
        </div>

        <div v-if="showTargetSelector" class="bg-base-200 rounded-box p-2">
          <EntitySelector
            v-if="target_type === 'application'"
            title="Applications"
            :fetch-fn="applicationsApi.list"
            :allow-create="false"
            @select="handleApplicationSelect"
            @cancel="showTargetSelector = false"
          />
          <EntitySelector
            v-else-if="target_type === 'service'"
            title="Services"
            :fetch-fn="servicesApi.list"
            :allow-create="false"
            @select="handleServiceSelect"
            @cancel="showTargetSelector = false"
          />
        </div>
        <div
          v-else-if="selectedName"
          class="flex items-center justify-between bg-base-200 rounded-box px-4 py-2"
        >
          <div class="flex items-center gap-2">
            <span class="badge badge-primary badge-sm">{{
              target_type === 'application' ? 'App' : 'Service'
            }}</span>
            <span class="font-medium">{{ selectedName }}</span>
          </div>
          <button
            type="button"
            class="btn btn-ghost btn-xs"
            @click="showTargetSelector = true"
          >
            Change
          </button>
        </div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Registrar</legend>
        <input v-model="form.registrar" type="text" class="input w-full" />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">DNS Provider</legend>
        <input v-model="form.dns_provider" type="text" class="input w-full" />
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Domain Expires</legend>
        <input v-model="form.expires_at" type="date" class="input w-full" />
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea v-model="form.notes" class="textarea w-full" rows="3" />
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ domain ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
