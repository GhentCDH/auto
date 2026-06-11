<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import type { Domain, CreateDomain, UpdateDomain } from '@/types';
import { requireConfig } from '@/composables/useConfig';
import EntityPicker from '../common/EntityPicker.vue';

const target_type = ref<'application' | 'service' | 'infra'>(
  requireConfig().defaults.domain.target_type as
    | 'application'
    | 'service'
    | 'infra'
);
const selectedName = ref<string | null>(null);

const props = defineProps<{
  domain?: Domain;
  initialName?: string;
  // When true, the parent supplies the target (e.g. InfraForm creating a domain
  // that targets the infra being created), so the target picker is hidden and
  // not required for validity.
  hideTarget?: boolean;
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
        target_infra_id: d.target_infra_id || undefined,
        notes: d.notes || '',
      };
      target_type.value = d.target_application_id
        ? 'application'
        : d.target_infra_id
          ? 'infra'
          : 'service';
      selectedName.value =
        d.target_application_name ||
        d.target_service_name ||
        d.target_infra_name ||
        null;
    }
  },
  { immediate: true }
);

const fqdnContainsProtocol = computed(() => form.value.fqdn.includes('://'));

const isValid = computed(() => {
  // Exactly one target of the three must be set.
  const count = [
    form.value.target_application_id,
    form.value.target_service_id,
    form.value.target_infra_id,
  ].filter((x) => x !== undefined).length;
  return (
    form.value.fqdn &&
    !fqdnContainsProtocol.value &&
    (props.hideTarget || count === 1)
  );
});

function handleSubmit() {
  if (isValid.value) {
    emit('submit', form.value);
  }
}

const nameInput = ref<HTMLInputElement>();

onMounted(() => {
  nameInput.value?.focus();
});

function selectTarget(
  type: 'application' | 'service' | 'infra',
  entity: { id: string; name: string }
) {
  form.value.target_application_id =
    type === 'application' ? entity.id : undefined;
  form.value.target_service_id = type === 'service' ? entity.id : undefined;
  form.value.target_infra_id = type === 'infra' ? entity.id : undefined;
  selectedName.value = entity.name;
}

// Switching target type clears the previous selection so the picker for the
// newly-chosen type opens fresh in search mode.
function switchTargetType() {
  form.value.target_application_id = undefined;
  form.value.target_service_id = undefined;
  form.value.target_infra_id = undefined;
  selectedName.value = null;
}

const handleApplicationSelect = (e: { id: string; name: string }) =>
  selectTarget('application', e);
const handleServiceSelect = (e: { id: string; name: string }) =>
  selectTarget('service', e);
const handleInfraSelect = (e: { id: string; name: string }) =>
  selectTarget('infra', e);
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
          :class="{ 'input-error': fqdnContainsProtocol }"
          placeholder="example.com"
          required
          autofocus
        />
        <p v-if="fqdnContainsProtocol" class="text-error text-sm mt-1">
          Domain name must not contain a protocol (e.g. https://)
        </p>
      </fieldset>

      <fieldset v-if="!hideTarget" class="fieldset col-span-2">
        <legend class="fieldset-legend">Target *</legend>
        <div class="flex gap-4 mb-2">
          <label class="label cursor-pointer gap-2">
            <input
              type="radio"
              name="target_type"
              value="application"
              v-model="target_type"
              class="radio radio-primary"
              @change="switchTargetType"
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
              @change="switchTargetType"
            />
            <span>Service</span>
          </label>
          <label class="label cursor-pointer gap-2">
            <input
              type="radio"
              name="target_type"
              value="infra"
              v-model="target_type"
              class="radio radio-primary"
              @change="switchTargetType"
            />
            <span>Infra</span>
          </label>
        </div>

        <EntityPicker
          v-if="target_type === 'application'"
          entity-type="application"
          :selected-name="selectedName"
          @select="handleApplicationSelect"
        />
        <EntityPicker
          v-else-if="target_type === 'service'"
          entity-type="service"
          :selected-name="selectedName"
          @select="handleServiceSelect"
        />
        <EntityPicker
          v-else-if="target_type === 'infra'"
          entity-type="infra"
          :selected-name="selectedName"
          @select="handleInfraSelect"
        />
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
      <button type="submit" class="btn btn-primary" :disabled="!isValid">
        {{ domain ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
