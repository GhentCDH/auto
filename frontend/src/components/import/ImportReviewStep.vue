<script setup lang="ts">
import { computed } from 'vue';
import type { ImportMapping, HealthcheckWithRelations } from '@/types';

const props = defineProps<{
  mappings: ImportMapping[];
  existingHealthchecks: HealthcheckWithRelations[];
  skippedDuplicateCount: number;
  importing: boolean;
}>();

const emit = defineEmits<{
  import: [];
  back: [];
  editMapping: [index: number];
}>();

const activeImports = computed(() =>
  props.mappings.filter((m) => !m.skip)
);

const skippedCount = computed(() =>
  props.mappings.filter((m) => m.skip).length
);

const newDomainsToCreate = computed(() => {
  const domains = new Set<string>();
  for (const m of activeImports.value) {
    if (m.domain_id === null) {
      domains.add(m.domain_fqdn);
    }
  }
  return Array.from(domains);
});

// Check for duplicates against existing healthchecks
const duplicates = computed(() => {
  const dupes: Array<{ mapping: ImportMapping; existing: HealthcheckWithRelations }> = [];

  for (const mapping of activeImports.value) {
    const monitor = mapping.monitor;
    // Build URL for comparison
    const importUrl = `${monitor.protocol}://${mapping.domain_fqdn}${monitor.path}`;

    for (const hc of props.existingHealthchecks) {
      const existingUrl = `${hc.protocol}://${hc.domain_fqdn}${hc.path}`;
      if (existingUrl === importUrl) {
        dupes.push({ mapping, existing: hc });
        break;
      }
    }
  }

  return dupes;
});
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="text-center">
      <h3 class="text-lg font-medium mb-2">Review Import</h3>
      <p class="text-base-content/70 text-sm">
        Review your mappings before importing
      </p>
    </div>

    <!-- Summary stats -->
    <div class="stats stats-horizontal shadow w-full">
      <div class="stat">
        <div class="stat-title">To Import</div>
        <div class="stat-value text-primary">{{ activeImports.length }}</div>
      </div>
      <div class="stat">
        <div class="stat-title">Skipped</div>
        <div class="stat-value text-base-content/50">{{ skippedCount }}</div>
      </div>
      <div v-if="skippedDuplicateCount > 0" class="stat">
        <div class="stat-title">Already Exist</div>
        <div class="stat-value text-base-content/50">{{ skippedDuplicateCount }}</div>
      </div>
      <div v-if="duplicates.length > 0" class="stat">
        <div class="stat-title">Duplicates</div>
        <div class="stat-value text-warning">{{ duplicates.length }}</div>
      </div>
    </div>

    <!-- Warnings -->
    <div v-if="skippedDuplicateCount > 0" class="alert text-sm">
      <span>
        {{ skippedDuplicateCount }} monitor{{
          skippedDuplicateCount !== 1 ? 's were' : ' was'
        }}
        automatically skipped because they already exist in the database.
      </span>
    </div>

    <div v-if="newDomainsToCreate.length > 0" class="alert alert-info text-sm">
      <span>
        {{ newDomainsToCreate.length }} new domain{{
          newDomainsToCreate.length !== 1 ? 's' : ''
        }}
        will be created:
        <span class="font-mono">{{ newDomainsToCreate.join(', ') }}</span>
      </span>
    </div>

    <div v-if="duplicates.length > 0" class="alert alert-warning text-sm">
      <span>
        {{ duplicates.length }} monitor{{
          duplicates.length !== 1 ? 's' : ''
        }}
        may be duplicates of existing healthchecks. They will still be imported.
      </span>
    </div>

    <!-- Import table -->
    <div class="overflow-x-auto">
      <table class="table table-sm">
        <thead>
          <tr>
            <th>Name</th>
            <th>Domain</th>
            <th>Healthcheck Target</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(mapping, index) in mappings"
            :key="mapping.monitor.kuma_id"
            :class="{ 'opacity-50': mapping.skip }"
          >
            <td>
              <div class="font-medium">{{ mapping.monitor.name }}</div>
              <div class="text-xs text-base-content/60 font-mono">
                {{ mapping.monitor.path }}
              </div>
            </td>
            <td>
              <span class="font-mono text-sm">{{ mapping.domain_fqdn }}</span>
              <template v-if="!mapping.skip && mapping.domain_id === null">
                <span class="badge badge-info badge-xs ml-1">new</span>
                <div class="text-xs text-base-content/60 mt-1">
                  <span class="badge badge-ghost badge-xs">
                    {{ mapping.domain_application_id ? 'App' : 'Service' }}
                  </span>
                  {{ mapping.domain_target_name }}
                </div>
              </template>
            </td>
            <td>
              <template v-if="!mapping.skip">
                <span class="badge badge-sm mr-1">
                  {{ mapping.application_id ? 'App' : 'Service' }}
                </span>
                {{ mapping.target_name }}
              </template>
              <span v-else class="text-base-content/50">—</span>
            </td>
            <td>
              <span v-if="mapping.skip" class="badge badge-ghost">Skipped</span>
              <span
                v-else-if="
                  duplicates.some(
                    (d) => d.mapping.monitor.kuma_id === mapping.monitor.kuma_id
                  )
                "
                class="badge badge-warning"
              >
                Duplicate?
              </span>
              <span v-else class="badge badge-success">Ready</span>
            </td>
            <td>
              <button
                type="button"
                class="btn btn-ghost btn-xs"
                @click="emit('editMapping', index)"
              >
                Edit
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Navigation -->
    <div class="flex justify-between gap-2 pt-4">
      <button
        type="button"
        class="btn btn-ghost"
        :disabled="importing"
        @click="emit('back')"
      >
        ← Back
      </button>
      <button
        type="button"
        class="btn btn-primary"
        :disabled="activeImports.length === 0 || importing"
        @click="emit('import')"
      >
        <span v-if="importing" class="loading loading-spinner loading-sm" />
        {{ importing ? 'Importing...' : `Import ${activeImports.length} Healthcheck${activeImports.length !== 1 ? 's' : ''}` }}
      </button>
    </div>
  </div>
</template>
