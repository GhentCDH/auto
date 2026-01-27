import type {
  Application,
  ApplicationWithRelations,
  CreateApplication,
  CreateDomain,
  CreateHost,
  CreateNetworkShare,
  CreateNote,
  CreatePerson,
  CreateStack,
  DashboardStats,
  Domain,
  DomainWithRelations,
  Host,
  HostWithRelations,
  LinkDomain,
  LinkHost,
  LinkNetworkShare,
  LinkPerson,
  NetworkShare,
  NetworkShareWithRelations,
  Note,
  PaginatedResponse,
  PaginationParams,
  Person,
  PersonWithRelations,
  SearchResults,
  Stack,
  StackWithRelations,
  UpdateApplication,
  UpdateDomain,
  UpdateHost,
  UpdateNetworkShare,
  UpdateNote,
  UpdatePerson,
  UpdateStack,
} from '@/types';

const BASE_URL = '/api';

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  if (!response.ok) {
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
  list: (params: PaginationParams = {}) =>
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
  linkHost: (appId: string, hostId: string, data: LinkHost = {}) =>
    request<void>(`/applications/${appId}/hosts/${hostId}`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  unlinkHost: (appId: string, hostId: string) =>
    request<void>(`/applications/${appId}/hosts/${hostId}`, {
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
};

// Hosts API
export const hostsApi = {
  list: (params: PaginationParams = {}) =>
    request<PaginatedResponse<Host>>(`/hosts${buildQueryString(params)}`),

  get: (id: string) => request<HostWithRelations>(`/hosts/${id}`),

  create: (data: CreateHost) =>
    request<Host>('/hosts', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateHost) =>
    request<Host>(`/hosts/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) => request<void>(`/hosts/${id}`, { method: 'DELETE' }),
};

// Domains API
export const domainsApi = {
  list: (params: PaginationParams = {}) =>
    request<PaginatedResponse<Domain>>(`/domains${buildQueryString(params)}`),

  get: (id: string) => request<DomainWithRelations>(`/domains/${id}`),

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
};

// People API
export const peopleApi = {
  list: (params: PaginationParams = {}) =>
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
  list: (params: PaginationParams = {}) =>
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
