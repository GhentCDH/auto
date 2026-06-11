import { computed } from 'vue';
import { useConfig } from '@/composables/useConfig';

export function toFilterOptions(
  obj: Record<string, string>
): { value: string; label: string }[] {
  return Object.entries(obj).map(([value, label]) => ({ value, label }));
}

const { config } = useConfig();

// Dropdown value -> label maps, sourced from the server config. The app gates
// rendering on config readiness (see App.vue), so these are populated before any
// consumer renders; `?? {}` only guards the brief pre-load window.
export const statuses = computed(() => config.value?.options.statuses ?? {});
export const environments = computed(
  () => config.value?.options.environments ?? {}
);
export const infraTypes = computed(
  () => config.value?.options.infra_types ?? {}
);
export const shareUsages = computed(
  () => config.value?.options.share_usages ?? {}
);
export const shareTypes = computed(
  () => config.value?.options.share_types ?? {}
);
export const domainTypes = computed(
  () => config.value?.options.domain_types ?? {}
);
export const domainStatus = computed(
  () => config.value?.options.domain_status ?? {}
);
export const contributionTypes = computed(
  () => config.value?.options.contribution_types ?? {}
);
export const noteTypes = computed(() => config.value?.options.note_types ?? {});

// Filter options for use in the ColumnFilter component
export const statusFilterOptions = computed(() =>
  toFilterOptions(statuses.value)
);
export const environmentFilterOptions = computed(() =>
  toFilterOptions(environments.value)
);
export const domainStatusFilterOptions = computed(() =>
  toFilterOptions(domainStatus.value)
);
export const shareTypeFilterOptions = computed(() =>
  toFilterOptions(shareTypes.value)
);
export const infraTypeFilterOptions = computed(() =>
  toFilterOptions(infraTypes.value)
);

// Not config-driven: a fixed boolean facet, not an entity option list.
export const personActiveFilterOptions = [
  { value: 'true', label: 'Active' },
  { value: 'false', label: 'Inactive' },
];
