<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import type {
  Infra,
  InfraIp,
  CreateInfra,
  UpdateInfra,
  NewInfraDomain,
  CreateDomain,
} from '@/types';
import { infraTypes } from '@/values';
import SelectWithCustom from '../common/SelectWithCustom.vue';
import EntityPicker from '../common/EntityPicker.vue';

const props = defineProps<{
  // Detail view passes InfraWithRelations (carries `ips` + targeting `domain`);
  // list passes nothing.
  infra?: Infra & {
    ips?: InfraIp[];
    domain?: { id: string; fqdn: string } | null;
  };
  initialName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateInfra | UpdateInfra];
  cancel: [];
}>();

const form = ref<CreateInfra>({
  name: props.initialName || '',
  description: '',
  type: 'server',
});

// IP source: a domain (server resolves IPs) or manually-entered IPs.
const ipMode = ref<'domain' | 'manual'>('manual');
const manualIps = ref<string[]>(['']);
const selectedDomainName = ref<string | null>(null);
// A new domain to create together with this infra (targets the infra itself,
// so it can't be created up front — the backend bundles it). Mutually
// exclusive with form.domain_id (picking an existing domain).
const pendingNewDomain = ref<NewInfraDomain | null>(null);

watch(
  () => props.infra,
  (inf) => {
    if (inf) {
      form.value = {
        name: inf.name,
        description: inf.description || '',
        type: inf.type,
      };
      // Prefill the targeting domain if present, else manual IPs.
      if (inf.domain) {
        ipMode.value = 'domain';
        form.value.domain_id = inf.domain.id;
        selectedDomainName.value = inf.domain.fqdn;
      } else {
        const manual = (inf.ips || []).filter((i) => i.source === 'manual');
        if (manual.length) {
          ipMode.value = 'manual';
          manualIps.value = manual.map((i) => i.ip);
        }
      }
    }
  },
  { immediate: true }
);

watch(
  () => form.value.name,
  (name) => {
    if (name.startsWith('gcdh')) form.value.type = 'vm';
  },
  { immediate: true }
);

function handleDomainSelect(domain: { id: string; name: string }) {
  form.value.domain_id = domain.id;
  pendingNewDomain.value = null;
  selectedDomainName.value = domain.name;
}

// User created a brand-new domain in the picker: hold it, don't persist yet.
// It's sent as `new_domain` on submit so the backend can target it at this infra.
function handleDomainCreate(data: unknown) {
  const d = data as CreateDomain;
  pendingNewDomain.value = {
    fqdn: d.fqdn,
    registrar: d.registrar || undefined,
    dns_provider: d.dns_provider || undefined,
    expires_at: d.expires_at || undefined,
    notes: d.notes || undefined,
  };
  form.value.domain_id = undefined;
  selectedDomainName.value = d.fqdn;
}

function addIpField() {
  manualIps.value.push('');
}
function removeIpField(i: number) {
  manualIps.value.splice(i, 1);
}

function handleSubmit() {
  const data: CreateInfra = {
    name: form.value.name,
    description: form.value.description,
    type: form.value.type,
  };
  if (ipMode.value === 'domain') {
    if (form.value.domain_id) data.domain_id = form.value.domain_id;
    else if (pendingNewDomain.value) data.new_domain = pendingNewDomain.value;
  } else {
    data.manual_ips = manualIps.value.map((s) => s.trim()).filter(Boolean);
  }
  emit('submit', data);
}

const nameInput = ref<HTMLInputElement>();
onMounted(() => {
  nameInput.value?.focus();
});
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <fieldset class="fieldset">
      <legend class="fieldset-legend">Name *</legend>
      <input
        v-model="form.name"
        type="text"
        ref="nameInput"
        class="input w-full"
        required
      />
    </fieldset>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Description</legend>
      <textarea v-model="form.description" class="textarea w-full" rows="3" />
      <div class="label">optional</div>
    </fieldset>

    <SelectWithCustom v-model="form.type" :options="infraTypes" allow-custom />

    <fieldset class="fieldset">
      <legend class="fieldset-legend">IP address</legend>
      <div class="flex gap-4 mb-2">
        <label class="label cursor-pointer gap-2">
          <input
            type="radio"
            value="domain"
            v-model="ipMode"
            class="radio radio-primary"
          />
          <span>From domain</span>
        </label>
        <label class="label cursor-pointer gap-2">
          <input
            type="radio"
            value="manual"
            v-model="ipMode"
            class="radio radio-primary"
          />
          <span>Manual</span>
        </label>
      </div>

      <!-- Domain: server resolves & refreshes IPs from DNS -->
      <template v-if="ipMode === 'domain'">
        <EntityPicker
          entity-type="domain"
          :selected-name="selectedDomainName"
          defer-create
          :create-form-props="{ hideTarget: true }"
          @select="handleDomainSelect"
          @create-deferred="handleDomainCreate"
        />
        <div class="label">
          IPs are resolved by the server and refreshed periodically.
        </div>
      </template>

      <!-- Manual: fixed IPs -->
      <template v-else>
        <div
          v-for="(_, i) in manualIps"
          :key="i"
          class="flex items-center gap-2 mb-2"
        >
          <input
            v-model="manualIps[i]"
            type="text"
            class="input w-full"
            placeholder="e.g. 157.193.244.99"
          />
          <button
            type="button"
            class="btn btn-ghost btn-sm"
            @click="removeIpField(i)"
          >
            ✕
          </button>
        </div>
        <button
          type="button"
          class="btn btn-sm btn-outline"
          @click="addIpField"
        >
          + Add IP
        </button>
      </template>
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary">
        {{ infra ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
