<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type {
  KumaMonitorImport,
  ImportMapping,
  HealthcheckWithRelations,
  CreateHealthcheck,
} from '@/types';
import ImportUploadStep from './ImportUploadStep.vue';
import ImportMapStep from './ImportMapStep.vue';
import ImportReviewStep from './ImportReviewStep.vue';
import { healthchecksApi, domainsApi } from '@/api';

const emit = defineEmits<{
  close: [];
  imported: [count: number];
  panelChange: [isOpen: boolean, widthRem: number];
}>();

type Step = 'upload' | 'map' | 'review' | 'complete';

const step = ref<Step>('upload');
const monitors = ref<KumaMonitorImport[]>([]);
const mappings = ref<ImportMapping[]>([]);
const currentIndex = ref(0);
const importing = ref(false);
const importResults = ref<{ success: number; failed: number }>({
  success: 0,
  failed: 0,
});
const importErrors = ref<string[]>([]);
const existingHealthchecks = ref<HealthcheckWithRelations[]>([]);
const skippedDuplicateCount = ref(0);

async function loadExistingHealthchecks() {
  try {
    const result = await healthchecksApi.list({ per_page: 1000 });
    existingHealthchecks.value = result.data;
  } catch {
    // Ignore errors - duplicate detection is optional
  }
}

function handleParsed(parsed: KumaMonitorImport[]) {
  // Build a set of existing healthcheck keys based on functional identity:
  // protocol + domain_fqdn + path + method + expected_status
  const existingKeys = new Set(
    existingHealthchecks.value.map(
      (hc) =>
        `${hc.protocol}://${hc.domain_fqdn}${hc.path}::${hc.method}::${hc.expected_status}`
    )
  );

  // Filter out duplicates and track skipped count
  const nonDuplicates: KumaMonitorImport[] = [];
  let skippedDuplicates = 0;

  for (const monitor of parsed) {
    const key = `${monitor.protocol}://${monitor.hostname}${monitor.path}::${monitor.method}::${monitor.expected_status}`;
    if (existingKeys.has(key)) {
      skippedDuplicates++;
    } else {
      nonDuplicates.push(monitor);
    }
  }

  monitors.value = nonDuplicates;
  skippedDuplicateCount.value = skippedDuplicates;
  mappings.value = [];
  currentIndex.value = 0;

  // If all monitors were duplicates, go to review with empty mappings
  if (nonDuplicates.length === 0) {
    step.value = 'review';
  } else {
    step.value = 'map';
  }
}

function handleSkipped() {
  step.value = 'review';
}

function handleMapped(mapping: ImportMapping) {
  // Store or update mapping
  if (currentIndex.value < mappings.value.length) {
    mappings.value[currentIndex.value] = mapping;
  } else {
    mappings.value.push(mapping);
  }

  // Move to next or review
  if (currentIndex.value + 1 < monitors.value.length) {
    currentIndex.value++;
  } else {
    step.value = 'review';
  }
}

function handleMapBack() {
  if (currentIndex.value > 0) {
    currentIndex.value--;
  } else {
    step.value = 'upload';
    monitors.value = [];
    mappings.value = [];
  }
}

function handleReviewBack() {
  // Go back to last monitor
  currentIndex.value = monitors.value.length - 1;
  step.value = 'map';
}

function handleEditMapping(index: number) {
  currentIndex.value = index;
  step.value = 'map';
}

async function handleImport() {
  importing.value = true;
  importResults.value = { success: 0, failed: 0 };
  importErrors.value = [];

  const activeImports = mappings.value.filter((m) => !m.skip);

  // Step 1: Create any missing domains
  const domainsToCreate = new Map<
    string,
    { id: string; target_application_id?: string; target_service_id?: string }
  >(); // fqdn -> id
  for (const mapping of activeImports) {
    if (mapping.domain_id === null) {
      // Use the domain-specific target fields (not the healthcheck target)
      if (mapping.domain_application_id) {
        domainsToCreate.set(mapping.domain_fqdn, {
          id: '',
          target_application_id: mapping.domain_application_id,
        });
      } else if (mapping.domain_service_id) {
        domainsToCreate.set(mapping.domain_fqdn, {
          id: '',
          target_service_id: mapping.domain_service_id,
        });
      }
    }
  }

  for (const fqdn of domainsToCreate.keys()) {
    try {
      const domain = domainsToCreate.get(fqdn);
      if (!domain) {
        continue;
      }
      const created = await domainsApi.create({
        fqdn,
        target_application_id: domain.target_application_id,
        target_service_id: domain.target_service_id,
      });

      domain.id = created.id;
    } catch (e) {
      const error = `Failed to create domain ${fqdn}: ${e instanceof Error ? e.message : 'Unknown error'}`;
      importErrors.value.push(error);
    }
  }

  // Step 2: Create healthchecks
  for (const mapping of activeImports) {
    const monitor = mapping.monitor;

    // Resolve domain ID
    let domainId = mapping.domain_id;
    if (domainId === null) {
      domainId = domainsToCreate.get(mapping.domain_fqdn)?.id || null;
    }

    if (!domainId) {
      importErrors.value.push(
        `Skipping ${monitor.name}: domain ${mapping.domain_fqdn} not available`
      );
      importResults.value.failed++;
      continue;
    }

    const healthcheck: CreateHealthcheck = {
      name: monitor.name,
      domain_id: domainId,
      application_id: mapping.application_id || undefined,
      service_id: mapping.service_id || undefined,
      protocol: monitor.protocol,
      path: monitor.path,
      method: monitor.method,
      expected_status: monitor.expected_status,
      timeout_seconds: monitor.timeout_seconds,
      headers: monitor.headers || undefined,
      expected_body: monitor.keyword || undefined,
      is_enabled: true,
      retry: monitor.retry,
      retry_interval: monitor.retry_interval,
      request_body: monitor.request_body || undefined,
      request_body_encoding: monitor.request_body_encoding,
      http_auth_user: monitor.http_auth_user || undefined,
      http_auth_pass: monitor.http_auth_pass || undefined,
    };

    try {
      await healthchecksApi.create(healthcheck);
      importResults.value.success++;
    } catch (e) {
      const error = `Failed to create ${monitor.name}: ${e instanceof Error ? e.message : 'Unknown error'}`;
      importErrors.value.push(error);
      importResults.value.failed++;
    }
  }

  importing.value = false;
  step.value = 'complete';
}

function handleDone() {
  emit('imported', importResults.value.success);
  emit('close');
}

onMounted(() => {
  loadExistingHealthchecks();
});
</script>

<template>
  <div class="p-4 max-h-[80vh] overflow-y-auto">
    <!-- Step indicators -->
    <ul class="steps steps-horizontal w-full mb-6">
      <li class="step step-primary">Upload</li>
      <li
        class="step"
        :class="{
          'step-primary':
            step === 'map' || step === 'review' || step === 'complete',
        }"
      >
        Map
      </li>
      <li class="step" :class="{ 'step-primary': step === 'complete' }">
        Import
      </li>
    </ul>

    <!-- Upload step -->
    <ImportUploadStep
      v-if="step === 'upload'"
      @parsed="handleParsed"
      @cancel="emit('close')"
    />

    <!-- Map step -->
    <ImportMapStep
      v-else-if="step === 'map'"
      :key="currentIndex"
      :monitor="monitors[currentIndex]"
      :index="currentIndex"
      :total="monitors.length"
      :existing-mapping="mappings[currentIndex]"
      @mapped="handleMapped"
      @skip="handleSkipped"
      @back="handleMapBack"
      @panel-change="
        (isOpen, widthRem) => emit('panelChange', isOpen, widthRem)
      "
    />

    <!-- Review step -->
    <ImportReviewStep
      v-else-if="step === 'review'"
      :mappings="mappings"
      :existing-healthchecks="existingHealthchecks"
      :skipped-duplicate-count="skippedDuplicateCount"
      :importing="importing"
      @import="handleImport"
      @back="handleReviewBack"
      @edit-mapping="handleEditMapping"
    />

    <!-- Complete step -->
    <div v-else-if="step === 'complete'" class="text-center space-y-4">
      <div class="text-6xl">
        {{ importResults.failed === 0 ? '✅' : '⚠️' }}
      </div>
      <h3 class="text-lg font-medium">Import Complete</h3>

      <div class="stats stats-horizontal shadow">
        <div class="stat">
          <div class="stat-title">Imported</div>
          <div class="stat-value text-success">{{ importResults.success }}</div>
        </div>
        <div v-if="importResults.failed > 0" class="stat">
          <div class="stat-title">Failed</div>
          <div class="stat-value text-error">{{ importResults.failed }}</div>
        </div>
      </div>

      <div
        v-if="importErrors.length > 0"
        class="text-left bg-base-200 rounded-box p-4 max-h-40 overflow-y-auto"
      >
        <p class="font-medium mb-2">Errors:</p>
        <ul class="text-sm text-error space-y-1">
          <li v-for="(error, i) in importErrors" :key="i">{{ error }}</li>
        </ul>
      </div>

      <button type="button" class="btn btn-primary" @click="handleDone">
        Done
      </button>
    </div>
  </div>
</template>
