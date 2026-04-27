export function toFilterOptions(
  obj: Record<string, string>
): { value: string; label: string }[] {
  return Object.entries(obj).map(([value, label]) => ({ value, label }));
}

export const statuses = {
  active: 'Active',
  inactive: 'Inactive',
  deprecated: 'Deprecated',
  archived: 'Archived',
};

export const environments = {
  prd: 'Production',
  dev: 'Development',
  qas: 'Quality Assurance',
  tst: 'Testing',
};

export const infraTypes = {
  nomad_cluster: 'Nomad Cluster',
  server: 'Server',
  vm: 'Virtual Machine',
};

export const shareUsages = {
  data: 'Data Storage',
  config: 'Configuration',
  logs: 'Logs',
  backup: 'Backup',
  media: 'Media',
};

export const shareTypes = {
  smb: 'SMB',
  nfs: 'NFS',
};

export const domainTypes = {
  A: 'A',
  AAAA: 'AAAA',
  CNAME: 'CNAME',
  MX: 'MX',
  TXT: 'TXT',
};

export const domainStatus = {
  active: 'Active',
  inactive: 'Inactive',
  expired: 'Expired',
};

export const contributionTypes = {
  project_owner: 'Project Owner',
  developer: 'Developer',
  maintainer: 'Maintainer',
  stakeholder: 'Stakeholder',
};

export const noteTypes = {
  general: 'General',
  documentation: 'Documentation',
  changelog: 'Changelog',
};

// Filter options for use in ColumnFilter component
export const statusFilterOptions = toFilterOptions(statuses);
export const environmentFilterOptions = toFilterOptions(environments);
export const domainStatusFilterOptions = toFilterOptions(domainStatus);
export const shareTypeFilterOptions = toFilterOptions(shareTypes);
export const infraTypeFilterOptions = toFilterOptions(infraTypes);

export const personActiveFilterOptions = [
  { value: 'true', label: 'Active' },
  { value: 'false', label: 'Inactive' },
];
