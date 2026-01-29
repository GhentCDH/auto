export const environments = {
  dev: 'Development',
  qas: 'Quality Assurance',
  prd: 'Production',
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
  developer: 'Developer',
  maintainer: 'Maintainer',
  communcation: 'Communication',
  stakeholder: 'Stakeholder',
  manager: 'Manager',
};

export const noteTypes = {
  general: 'General',
  documentation: 'Documentation',
  changelog: 'Changelog',
};

// Filter options for use in ColumnFilter component
export const statusFilterOptions = [
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
  { value: 'deprecated', label: 'Deprecated' },
];

export const environmentFilterOptions = [
  { value: 'prd', label: 'Production' },
  { value: 'qas', label: 'QAS' },
  { value: 'dev', label: 'Development' },
  { value: 'tst', label: 'Testing' },
];

export const domainStatusFilterOptions = [
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
  { value: 'expired', label: 'Expired' },
];

export const personActiveFilterOptions = [
  { value: 'true', label: 'Active' },
  { value: 'false', label: 'Inactive' },
];

export const shareTypeFilterOptions = [
  { value: 'smb', label: 'SMB' },
  { value: 'nfs', label: 'NFS' },
];

export const infraTypeFilterOptions = [
  { value: 'nomad_cluster', label: 'Nomad Cluster' },
  { value: 'server', label: 'Server' },
  { value: 'vm', label: 'Virtual Machine' },
];
