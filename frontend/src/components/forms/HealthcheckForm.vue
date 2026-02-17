<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import type {
  Healthcheck,
  CreateHealthcheck,
  UpdateHealthcheck,
} from '@/types';
import EntitySelector from '../common/EntitySelector.vue';
import { applicationsApi, servicesApi, domainsApi } from '@/api';

const target_type = ref<'application' | 'service'>('application');
const showTargetSelector = ref(false);
const showDomainSelector = ref(false);
const selectedTargetName = ref<string | null>(null);
const selectedDomainName = ref<string | null>(null);

const props = defineProps<{
  healthcheck?: Healthcheck;
  initialName?: string;
  initialApplicationId?: string;
  initialServiceId?: string;
  initialTargetName?: string;
}>();

const emit = defineEmits<{
  submit: [data: CreateHealthcheck | UpdateHealthcheck];
  cancel: [];
}>();

if (props.initialApplicationId) {
  target_type.value = 'application';
} else if (props.initialServiceId) {
  target_type.value = 'service';
}

if (props.initialTargetName) {
  selectedTargetName.value = props.initialTargetName;
}

const form = ref<CreateHealthcheck>({
  name: props.initialName || '',
  application_id: props.initialApplicationId,
  service_id: props.initialServiceId,
  domain_id: '',
  protocol: 'https',
  path: '/',
  method: 'GET',
  headers: '',
  expected_status: 200,
  expected_body: '',
  timeout_seconds: 30,
  is_enabled: true,
  notes: '',
});

// Parse headers from JSON string for editing
const headersArray = ref<Array<{ key: string; value: string }>>([]);

function parseHeaders(headersJson: string | null | undefined) {
  if (!headersJson) return [];
  try {
    const parsed = JSON.parse(headersJson);
    return Object.entries(parsed).map(([key, value]) => ({
      key,
      value: String(value),
    }));
  } catch {
    return [];
  }
}

function headersToJson(
  headers: Array<{ key: string; value: string }>
): string | undefined {
  const filtered = headers.filter((h) => h.key.trim());
  if (filtered.length === 0) return undefined;
  const obj: Record<string, string> = {};
  for (const h of filtered) {
    obj[h.key] = h.value;
  }
  return JSON.stringify(obj);
}

watch(
  () => props.healthcheck,
  (hc) => {
    if (hc) {
      form.value = {
        name: hc.name,
        application_id: hc.application_id || undefined,
        service_id: hc.service_id || undefined,
        domain_id: hc.domain_id,
        protocol: hc.protocol,
        path: hc.path,
        method: hc.method,
        headers: hc.headers || '',
        expected_status: hc.expected_status,
        expected_body: hc.expected_body || '',
        timeout_seconds: hc.timeout_seconds,
        is_enabled: hc.is_enabled,
        notes: hc.notes || '',
      };
      target_type.value = hc.application_id ? 'application' : 'service';
      headersArray.value = parseHeaders(hc.headers);
    }
  },
  { immediate: true }
);

// Initialize target type based on initial props
onMounted(() => {
  if (props.initialServiceId) {
    target_type.value = 'service';
    form.value.service_id = props.initialServiceId;
    form.value.application_id = undefined;
  } else if (props.initialApplicationId) {
    target_type.value = 'application';
    form.value.application_id = props.initialApplicationId;
    form.value.service_id = undefined;
  }
  nameInput.value?.focus();
});

const isValid = computed(() => {
  const hasTarget =
    (form.value.application_id && !form.value.service_id) ||
    (!form.value.application_id && form.value.service_id);
  return form.value.name && hasTarget && form.value.domain_id;
});

function handleSubmit() {
  if (!isValid.value) return;

  const headers = headersToJson(headersArray.value);
  emit('submit', {
    ...form.value,
    headers,
    expected_body: form.value.expected_body || undefined,
    notes: form.value.notes || undefined,
  });
}

const nameInput = ref<HTMLInputElement>();

function handleApplicationSelect(application: { id: string; name: string }) {
  form.value.service_id = undefined;
  form.value.application_id = application.id;
  showTargetSelector.value = false;
  selectedTargetName.value = application.name;
}

function handleServiceSelect(service: { id: string; name: string }) {
  form.value.service_id = service.id;
  form.value.application_id = undefined;
  showTargetSelector.value = false;
  selectedTargetName.value = service.name;
}

function handleDomainSelect(domain: { id: string; name: string }) {
  form.value.domain_id = domain.id;
  showDomainSelector.value = false;
  selectedDomainName.value = domain.name;
}

function addHeader() {
  headersArray.value.push({ key: '', value: '' });
}

function removeHeader(index: number) {
  headersArray.value.splice(index, 1);
}
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Name *</legend>
        <input
          v-model="form.name"
          ref="nameInput"
          type="text"
          class="input w-full"
          placeholder="e.g., Production Health Check"
          required
        />
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
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
                form.service_id = undefined;
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
                form.application_id = undefined;
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
        <div v-else-if="selectedTargetName" class="text-sm text-right">
          Selected: <span class="font-medium">{{ selectedTargetName }}</span>
          <button
            type="button"
            class="btn btn-ghost btn-xs ml-2"
            @click="showTargetSelector = true"
          >
            Change
          </button>
        </div>
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Domain *</legend>
        <button
          v-if="!showDomainSelector && !selectedDomainName"
          type="button"
          class="btn btn-outline w-full"
          @click="showDomainSelector = true"
        >
          Select Domain
        </button>
        <div v-else-if="showDomainSelector" class="bg-base-200 rounded-box p-2">
          <EntitySelector
            title="Domains"
            :fetch-fn="domainsApi.list"
            :allow-create="false"
            @select="handleDomainSelect"
            @cancel="showDomainSelector = false"
          />
        </div>
        <div v-else class="text-sm text-right">
          Selected: <span class="font-medium">{{ selectedDomainName }}</span>
          <button
            type="button"
            class="btn btn-ghost btn-xs ml-2"
            @click="showDomainSelector = true"
          >
            Change
          </button>
        </div>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Protocol</legend>
        <select v-model="form.protocol" class="select w-full">
          <option value="https">HTTPS</option>
          <option value="http">HTTP</option>
        </select>
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Method</legend>
        <select v-model="form.method" class="select w-full">
          <option value="GET">GET</option>
          <option value="HEAD">HEAD</option>
          <option value="POST">POST</option>
        </select>
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Path</legend>
        <input
          v-model="form.path"
          type="text"
          class="input w-full"
          placeholder="/"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Expected Status</legend>
        <input
          v-model.number="form.expected_status"
          type="number"
          class="input w-full"
          min="100"
          max="599"
        />
      </fieldset>

      <fieldset class="fieldset">
        <legend class="fieldset-legend">Timeout (seconds)</legend>
        <input
          v-model.number="form.timeout_seconds"
          type="number"
          class="input w-full"
          min="1"
          max="300"
        />
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Expected Body (substring match)</legend>
        <input
          v-model="form.expected_body"
          type="text"
          class="input w-full"
          placeholder="Optional - leave empty to skip body check"
        />
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Custom Headers</legend>
        <div class="space-y-2">
          <div
            v-for="(header, index) in headersArray"
            :key="index"
            class="flex gap-2"
          >
            <input
              v-model="header.key"
              type="text"
              class="input input-sm flex-1"
              placeholder="Header name"
            />
            <input
              v-model="header.value"
              type="text"
              class="input input-sm flex-1"
              placeholder="Header value"
            />
            <button
              type="button"
              class="btn btn-ghost btn-sm btn-square"
              @click="removeHeader(index)"
            >
              &times;
            </button>
          </div>
          <button type="button" class="btn btn-ghost btn-sm" @click="addHeader">
            + Add Header
          </button>
        </div>
      </fieldset>

      <fieldset class="fieldset md:col-span-2">
        <legend class="fieldset-legend">Enabled</legend>
        <label class="label cursor-pointer justify-start gap-4">
          <input
            type="checkbox"
            v-model="form.is_enabled"
            class="toggle toggle-primary"
          />
          <span>{{ form.is_enabled ? 'Enabled' : 'Disabled' }}</span>
        </label>
      </fieldset>
    </div>

    <fieldset class="fieldset">
      <legend class="fieldset-legend">Notes</legend>
      <textarea v-model="form.notes" class="textarea w-full" rows="3" />
    </fieldset>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
      <button type="submit" class="btn btn-primary" :disabled="!isValid">
        {{ healthcheck ? 'Update' : 'Create' }}
      </button>
    </div>
  </form>
</template>
