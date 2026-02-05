<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { healthchecksApi } from '@/api';
import type {
  HealthcheckWithRelations,
  HealthcheckExecuteResult,
} from '@/types';
import EntityDetail from '@/components/common/EntityDetail.vue';
import StatusBadge from '@/components/common/StatusBadge.vue';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';

const router = useRouter();

const executeLoading = ref(false);
const executeResult = ref<HealthcheckExecuteResult | null>(null);
const executeError = ref('');

const entityDetailRef = ref<InstanceType<typeof EntityDetail> | null>(null);

const healthcheck = computed(() => entityDetailRef.value?.entity as HealthcheckWithRelations | null);

async function executeHealthcheck() {
  if (!healthcheck.value) return;
  executeLoading.value = true;
  executeError.value = '';
  executeResult.value = null;
  try {
    executeResult.value = await healthchecksApi.execute(healthcheck.value.id);
  } catch (e) {
    executeError.value = e instanceof Error ? e.message : 'Execute failed';
  } finally {
    executeLoading.value = false;
  }
}

function buildUrl(hc: HealthcheckWithRelations): string {
  return `${hc.protocol}://${hc.domain_fqdn}${hc.path}`;
}

function formatHeaders(
  headers: Record<string, string> | null
): Array<{ key: string; value: string }> {
  if (!headers) return [];
  return Object.entries(headers).map(([key, value]) => ({ key, value }));
}
</script>

<template>
  <EntityDetail
    ref="entityDetailRef"
    entity-name="Healthcheck"
    list-path="/healthchecks"
    :fetch-fn="
      healthchecksApi.get as (id: string) => Promise<{ id: string; name: string }>
    "
    :update-fn="
      healthchecksApi.update as (id: string, data: unknown) => Promise<unknown>
    "
    :delete-fn="healthchecksApi.delete"
  >
    <template #status="{ entity }">
      <StatusBadge
        :status="(entity as HealthcheckWithRelations).is_enabled ? 'active' : 'inactive'"
      />
    </template>

    <template #details="{ entity }">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <div class="text-sm text-base-content/70">URL</div>
          <div class="font-mono text-sm break-all">
            {{ buildUrl(entity as HealthcheckWithRelations) }}
          </div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Method</div>
          <div>
            <span class="badge badge-outline">{{
              (entity as HealthcheckWithRelations).method
            }}</span>
          </div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Expected Status</div>
          <div>{{ (entity as HealthcheckWithRelations).expected_status }}</div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Timeout</div>
          <div>
            {{ (entity as HealthcheckWithRelations).timeout_seconds }} seconds
          </div>
        </div>
        <div v-if="(entity as HealthcheckWithRelations).expected_body">
          <div class="text-sm text-base-content/70">Expected Body</div>
          <div class="font-mono text-sm">
            {{ (entity as HealthcheckWithRelations).expected_body }}
          </div>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Target</div>
          <router-link
            v-if="(entity as HealthcheckWithRelations).application_id"
            :to="`/applications/${(entity as HealthcheckWithRelations).application_id}`"
            class="link link-hover"
          >
            {{ (entity as HealthcheckWithRelations).application_name }}
          </router-link>
          <router-link
            v-else-if="(entity as HealthcheckWithRelations).service_id"
            :to="`/services/${(entity as HealthcheckWithRelations).service_id}`"
            class="link link-hover"
          >
            {{ (entity as HealthcheckWithRelations).service_name }}
          </router-link>
        </div>
        <div>
          <div class="text-sm text-base-content/70">Domain</div>
          <router-link
            :to="`/domains/${(entity as HealthcheckWithRelations).domain_id}`"
            class="link link-hover"
          >
            {{ (entity as HealthcheckWithRelations).domain_fqdn }}
          </router-link>
        </div>
      </div>

      <!-- Custom Headers -->
      <div
        v-if="
          (entity as HealthcheckWithRelations).parsed_headers &&
          Object.keys((entity as HealthcheckWithRelations).parsed_headers || {})
            .length > 0
        "
        class="mt-4"
      >
        <div class="text-sm text-base-content/70 mb-2">Custom Headers</div>
        <div class="overflow-x-auto">
          <table class="table table-xs">
            <thead>
              <tr>
                <th>Header</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="h in formatHeaders(
                  (entity as HealthcheckWithRelations).parsed_headers
                )"
                :key="h.key"
              >
                <td class="font-mono">{{ h.key }}</td>
                <td class="font-mono">{{ h.value }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Notes -->
      <div v-if="(entity as HealthcheckWithRelations).notes" class="mt-4">
        <div class="text-sm text-base-content/70">Notes</div>
        <div>{{ (entity as HealthcheckWithRelations).notes }}</div>
      </div>
    </template>

    <template #relations="{ entity }">
      <h2 class="card-title mb-4">Execute Healthcheck</h2>

      <button
        class="btn btn-primary w-full mb-4"
        :disabled="executeLoading"
        @click="executeHealthcheck"
      >
        <span
          v-if="executeLoading"
          class="loading loading-spinner loading-sm"
        />
        {{ executeLoading ? 'Executing...' : 'Run Healthcheck' }}
      </button>

      <!-- Execute Result -->
      <div v-if="executeResult" class="space-y-4">
        <div
          class="alert"
          :class="executeResult.success ? 'alert-success' : 'alert-error'"
        >
          <div>
            <div class="font-bold">
              {{ executeResult.success ? 'Success' : 'Failed' }}
            </div>
            <div class="text-sm">
              <span v-if="executeResult.status_code">
                Status: {{ executeResult.status_code }}
              </span>
              <span v-if="executeResult.error">
                Error: {{ executeResult.error }}
              </span>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <div class="text-base-content/70">Response Time</div>
            <div>{{ executeResult.response_time_ms }}ms</div>
          </div>
          <div v-if="executeResult.body_match !== null">
            <div class="text-base-content/70">Body Match</div>
            <div>
              <span
                class="badge"
                :class="executeResult.body_match ? 'badge-success' : 'badge-error'"
              >
                {{ executeResult.body_match ? 'Matched' : 'No Match' }}
              </span>
            </div>
          </div>
          <div class="col-span-2">
            <div class="text-base-content/70">URL Tested</div>
            <div class="font-mono text-xs break-all">{{ executeResult.url }}</div>
          </div>
          <div class="col-span-2">
            <div class="text-base-content/70">Executed At</div>
            <div>{{ new Date(executeResult.executed_at).toLocaleString() }}</div>
          </div>
        </div>
      </div>

      <!-- Execute Error -->
      <div v-if="executeError" class="alert alert-error mt-4">
        {{ executeError }}
      </div>
    </template>

    <template #form="{ entity, onSubmit, onCancel }">
      <HealthcheckForm
        :healthcheck="entity as HealthcheckWithRelations"
        @submit="onSubmit"
        @cancel="onCancel"
      />
    </template>
  </EntityDetail>
</template>
