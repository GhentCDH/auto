<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import type { Infra, InfraIp, CreateInfra, UpdateInfra } from '@/types';
import { infraTypes } from '@/values';
import { domainsApi } from '@/api';
import SelectWithCustom from '../common/SelectWithCustom.vue';
import EntitySelector from '../common/EntitySelector.vue';

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
const showDomainSelector = ref(false);

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
  selectedDomainName.value = domain.name;
  showDomainSelector.value = false;
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
        <div
          v-if="selectedDomainName && !showDomainSelector"
          class="flex items-center justify-between bg-base-200 rounded-box px-4 py-2"
        >
          <div class="flex items-center gap-2">
            <span class="badge badge-primary badge-sm">Domain</span>
            <span class="font-medium">{{ selectedDomainName }}</span>
          </div>
          <button
            type="button"
            class="btn btn-ghost btn-xs"
            @click="showDomainSelector = true"
          >
            Change
          </button>
        </div>
        <div v-else class="bg-base-200 rounded-box p-2">
          <EntitySelector
            title="Domains"
            :fetch-fn="domainsApi.list"
            :allow-create="false"
            @select="handleDomainSelect"
            @cancel="showDomainSelector = false"
          />
        </div>
        <div class="label">IPs are resolved by the server and refreshed periodically.</div>
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
        <button type="button" class="btn btn-sm btn-outline" @click="addIpField">
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
