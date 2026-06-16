import type {
  Application,
  ApplicationFilterParams,
  ApplicationWithRelations,
  AuthUser,
  PublicConfig,
  Role,
  ResetRequestSummary,
  SetupLink,
  UserSummary,
  CreateApplication,
  CreateDomain,
  CreateHealthcheck,
  CreateInfra,
  CreateNetworkShare,
  CreateNote,
  CreatePerson,
  CreateService,
  CreateStack,
  DashboardStats,
  DnsLookup,
  Domain,
  DomainFilterParams,
  DomainWithRelations,
  Healthcheck,
  HealthcheckExecuteResult,
  HealthcheckFilterParams,
  HealthcheckWithRelations,
  Infra,
  InfraFilterParams,
  InfraLoad,
  InfraWithRelations,
  LoadPoint,
  KumaEndpoint,
  KumaMonitor,
  LinkDomain,
  LinkInfra,
  LinkNetworkShare,
  LinkPerson,
  LinkService,
  NetworkShare,
  NetworkShareWithRelations,
  Note,
  PaginatedResponse,
  PaginationParams,
  Person,
  PersonFilterParams,
  PersonWithRelations,
  SearchResults,
  Service,
  ServiceFilterParams,
  ServiceWithRelations,
  ShareFilterParams,
  Stack,
  StackWithRelations,
  UpdateApplication,
  UpdateDomain,
  UpdateHealthcheck,
  UpdateInfra,
  UpdateNetworkShare,
  UpdateNote,
  UpdatePerson,
  UpdateService,
  UpdateStack,
} from '@/types';

const BASE_URL = '/api';

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`, {
    // Send the session cookie with every request (same-origin in production,
    // and through the Vite dev proxy in development).
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  if (!response.ok) {
    // A 401 on a normal request means the session expired or is missing; bounce
    // to the login page. Auth endpoints (e.g. /auth/me probing) handle their own
    // 401s, so they are exempt to avoid redirect loops.
    if (
      response.status === 401 &&
      !url.startsWith('/auth/') &&
      window.location.pathname !== '/login'
    ) {
      window.location.assign('/login');
    }
    const error = await response
      .json()
      .catch(() => ({ message: response.statusText }));
    throw new Error(error.message || 'Request failed');
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

function buildQueryString(params: any): string {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') {
      searchParams.append(key, String(value));
    }
  }
  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : '';
}

// Applications API
export const applicationsApi = {
  list: (params: ApplicationFilterParams = {}) =>
    request<PaginatedResponse<Application>>(
      `/applications${buildQueryString(params)}`
    ),

  get: (id: string) => request<ApplicationWithRelations>(`/applications/${id}`),

  create: (data: CreateApplication) =>
    request<Application>('/applications', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateApplication) =>
    request<Application>(`/applications/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    request<void>(`/applications/${id}`, { method: 'DELETE' }),

  // Relationship management
  linkInfra: (appId: string, infraId: string, data: LinkInfra = {}) =>
    request<void>(`/applications/${appId}/infra/${infraId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkInfra: (appId: string, infraId: string) =>
    request<void>(`/applications/${appId}/infra/${infraId}`, {
      method: 'DELETE',
    }),

  linkService: (appId: string, serviceId: string, data: LinkService = {}) =>
    request<void>(`/applications/${appId}/services/${serviceId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkService: (appId: string, serviceId: string) =>
    request<void>(`/applications/${appId}/services/${serviceId}`, {
      method: 'DELETE',
    }),

  linkDomain: (appId: string, domainId: string, data: LinkDomain = {}) =>
    request<void>(`/applications/${appId}/domains/${domainId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkDomain: (appId: string, domainId: string) =>
    request<void>(`/applications/${appId}/domains/${domainId}`, {
      method: 'DELETE',
    }),

  linkPerson: (appId: string, personId: string, data: LinkPerson = {}) =>
    request<void>(`/applications/${appId}/people/${personId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkPerson: (appId: string, personId: string) =>
    request<void>(`/applications/${appId}/people/${personId}`, {
      method: 'DELETE',
    }),

  linkShare: (appId: string, shareId: string, data: LinkNetworkShare = {}) =>
    request<void>(`/applications/${appId}/shares/${shareId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkShare: (appId: string, shareId: string) =>
    request<void>(`/applications/${appId}/shares/${shareId}`, {
      method: 'DELETE',
    }),

  linkStack: (appId: string, stackId: string) =>
    request<void>(`/applications/${appId}/stacks/${stackId}`, {
      method: 'POST',
    }),

  unlinkStack: (appId: string, stackId: string) =>
    request<void>(`/applications/${appId}/stacks/${stackId}`, {
      method: 'DELETE',
    }),

  syncOutline: (id: string) =>
    request<void>(`/applications/${id}/sync-outline`, { method: 'POST' }),
};

// Services API
export const servicesApi = {
  list: (params: ServiceFilterParams = {}) =>
    request<PaginatedResponse<Service>>(`/services${buildQueryString(params)}`),

  get: (id: string) => request<ServiceWithRelations>(`/services/${id}`),

  create: (data: CreateService) =>
    request<Service>('/services', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateService) =>
    request<Service>(`/services/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    request<void>(`/services/${id}`, { method: 'DELETE' }),

  // Relationship management
  linkInfra: (serviceId: string, infraId: string, data: LinkInfra = {}) =>
    request<void>(`/services/${serviceId}/infra/${infraId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkInfra: (serviceId: string, infraId: string) =>
    request<void>(`/services/${serviceId}/infra/${infraId}`, {
      method: 'DELETE',
    }),

  syncOutline: (id: string) =>
    request<void>(`/services/${id}/sync-outline`, { method: 'POST' }),
};

// Infra API
export const infraApi = {
  list: (params: InfraFilterParams = {}) =>
    request<PaginatedResponse<Infra>>(`/infra${buildQueryString(params)}`),

  get: (id: string) => request<InfraWithRelations>(`/infra/${id}`),

  create: (data: CreateInfra) =>
    request<Infra>('/infra', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateInfra) =>
    request<Infra>(`/infra/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/infra/${id}`, { method: 'DELETE' }),

  // Re-resolve infra IPs from their domains (forced; bypasses staleness check).
  syncAll: () => request<void>('/infra/sync', { method: 'POST' }),
  syncOne: (id: string) =>
    request<InfraWithRelations>(`/infra/${id}/sync`, { method: 'POST' }),

  // Zabbix load: bulk current load per infra (only Zabbix-matched infra), and
  // per-infra history. Both empty when Zabbix is unconfigured / unmatched.
  loads: () => request<Record<string, InfraLoad>>('/infra/loads'),
  loadHistory: (id: string, hours = 24) =>
    request<LoadPoint[]>(`/infra/${id}/load/history?hours=${hours}`),
};

// Domains API
export const domainsApi = {
  list: async (params: DomainFilterParams = {}) => {
    const response = await request<PaginatedResponse<DomainWithRelations>>(
      `/domains${buildQueryString(params)}`
    );
    return {
      ...response,
      data: response.data.map((domain) => ({
        ...domain,
        name: domain.fqdn,
      })),
    };
  },

  get: async (id: string) => {
    const response = await request<DomainWithRelations>(`/domains/${id}`);
    return {
      ...response,
      name: response.fqdn,
    };
  },

  create: (data: CreateDomain) =>
    request<Domain>('/domains', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateDomain) =>
    request<Domain>(`/domains/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/domains/${id}`, { method: 'DELETE' }),

  // Live DNS records — resolved server-side on demand, never persisted.
  getDns: (id: string) => request<DnsLookup>(`/domains/${id}/dns`),
  getAllDns: () => request<DnsLookup[]>(`/domains/dns`),
};

// People API
export const peopleApi = {
  list: (params: PersonFilterParams = {}) =>
    request<PaginatedResponse<Person>>(`/people${buildQueryString(params)}`),

  get: (id: string) => request<PersonWithRelations>(`/people/${id}`),

  create: (data: CreatePerson) =>
    request<Person>('/people', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdatePerson) =>
    request<Person>(`/people/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/people/${id}`, { method: 'DELETE' }),
};

// Network Shares API
export const sharesApi = {
  list: (params: ShareFilterParams = {}) =>
    request<PaginatedResponse<NetworkShare>>(
      `/shares${buildQueryString(params)}`
    ),

  get: (id: string) => request<NetworkShareWithRelations>(`/shares/${id}`),

  create: (data: CreateNetworkShare) =>
    request<NetworkShare>('/shares', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateNetworkShare) =>
    request<NetworkShare>(`/shares/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/shares/${id}`, { method: 'DELETE' }),
};

// Notes API
export const notesApi = {
  list: (entityType: string, entityId: string, params: PaginationParams = {}) =>
    request<PaginatedResponse<Note>>(
      `/notes${buildQueryString({ entity_type: entityType, entity_id: entityId, ...params })}`
    ),

  get: (id: string) => request<Note>(`/notes/${id}`),

  create: (data: CreateNote) =>
    request<Note>('/notes', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateNote) =>
    request<Note>(`/notes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/notes/${id}`, { method: 'DELETE' }),
};

// Dashboard API
export const dashboardApi = {
  getStats: () => request<DashboardStats>('/dashboard/stats'),
};

// Stacks API
export const stacksApi = {
  list: (params: PaginationParams = {}) =>
    request<PaginatedResponse<Stack>>(`/stacks${buildQueryString(params)}`),

  get: (id: string) => request<StackWithRelations>(`/stacks/${id}`),

  create: (data: CreateStack) =>
    request<Stack>('/stacks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateStack) =>
    request<Stack>(`/stacks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/stacks/${id}`, { method: 'DELETE' }),
};

// Search API
export const searchApi = {
  search: (query: string) =>
    request<SearchResults>(`/search${buildQueryString({ q: query })}`),
};

// Healthchecks API
export const healthchecksApi = {
  list: (params: HealthcheckFilterParams = {}) =>
    request<PaginatedResponse<HealthcheckWithRelations>>(
      `/healthchecks${buildQueryString(params)}`
    ),

  get: (id: string) => request<HealthcheckWithRelations>(`/healthchecks/${id}`),

  create: (data: CreateHealthcheck) =>
    request<Healthcheck>('/healthchecks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateHealthcheck) =>
    request<Healthcheck>(`/healthchecks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    request<void>(`/healthchecks/${id}`, { method: 'DELETE' }),

  execute: (id: string) =>
    request<HealthcheckExecuteResult>(`/healthchecks/${id}/execute`),

  exportKuma: () => request<KumaMonitor[]>('/healthchecks/export/kuma'),

  kumaEndpoint: () => request<KumaEndpoint>('/healthchecks/kuma-endpoint'),

  syncKumaAll: () =>
    request<void>('/healthchecks/sync/kuma', { method: 'POST' }),

  syncKumaOne: (id: string) =>
    request<void>(`/healthchecks/sync/kuma/${id}`, { method: 'POST' }),
};

// Resolve API
export const resolveApi = {
  resolve: (id: string) =>
    request<{ id: string; name: string; entity_type: string }>(
      `/resolve/${id}`
    ),
};

// Version API
export const versionApi = {
  get: () => request<{ version: string }>('/version'),
};

// Config API — server-provided form defaults and dropdown options
export const configApi = {
  get: () => request<PublicConfig>('/config'),
};

// Outline Sync API
export const outlineApi = {
  sync: () => request<void>('/outline/sync'),
};

// Authentication API
export const authApi = {
  login: (username: string, password: string) =>
    request<AuthUser>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),
  logout: () => request<void>('/auth/logout', { method: 'POST' }),
  me: () => request<AuthUser>('/auth/me'),
  changePassword: (current_password: string, new_password: string) =>
    request<void>('/auth/change-password', {
      method: 'POST',
      body: JSON.stringify({ current_password, new_password }),
    }),
  setPassword: (token: string, password: string) =>
    request<void>('/auth/set-password', {
      method: 'POST',
      body: JSON.stringify({ token, password }),
    }),
  resetRequest: (username: string) =>
    request<void>('/auth/reset-request', {
      method: 'POST',
      body: JSON.stringify({ username }),
    }),
  link: (link_token: string, username: string, password: string) =>
    request<AuthUser>('/auth/link', {
      method: 'POST',
      body: JSON.stringify({ link_token, username, password }),
    }),
  // OIDC login is a full-page redirect, not a fetch.
  oidcStartUrl: '/api/auth/oidc/start',
};

// Admin user management API
export const usersApi = {
  list: () => request<UserSummary[]>('/admin/users'),
  create: (username: string, email: string | null, role: Role) =>
    request<UserSummary>('/admin/users', {
      method: 'POST',
      body: JSON.stringify({ username, email, role }),
    }),
  remove: (id: string) =>
    request<void>(`/admin/users/${id}`, { method: 'DELETE' }),
  updateRole: (id: string, role: Role) =>
    request<void>(`/admin/users/${id}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    }),
  setupLink: (id: string) =>
    request<SetupLink>(`/admin/users/${id}/setup-link`, { method: 'POST' }),
  revokeSessions: (id: string) =>
    request<void>(`/admin/users/${id}/revoke-sessions`, { method: 'POST' }),
  resetRequests: () =>
    request<ResetRequestSummary[]>('/admin/reset-requests'),
};
