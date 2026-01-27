// Base types
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface PaginationParams {
  page?: number;
  per_page?: number;
  search?: string;
}

// Application types
export interface Application {
  id: string;
  name: string;
  description: string | null;
  repository_url: string | null;
  status: string;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateApplication {
  name: string;
  description?: string;
  repository_url?: string;
  status?: string;
}

export interface UpdateApplication {
  name?: string;
  description?: string;
  repository_url?: string;
  status?: string;
}

export interface ApplicationWithRelations extends Application {
  hosts: HostRelation[];
  domains: DomainRelation[];
  people: PersonRelation[];
  network_shares: NetworkShareRelation[];
  notes: Note[];
  stacks: StackRelation[];
}

// Host types
export interface Host {
  id: string;
  name: string;
  host_type: string;
  hostname: string | null;
  ip_address: string | null;
  location: string | null;
  os: string | null;
  specs: string | null;
  status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateHost {
  name: string;
  host_type: string;
  hostname?: string;
  ip_address?: string;
  location?: string;
  os?: string;
  specs?: string;
  status?: string;
  notes?: string;
}

export interface UpdateHost {
  name?: string;
  host_type?: string;
  hostname?: string;
  ip_address?: string;
  location?: string;
  os?: string;
  specs?: string;
  status?: string;
  notes?: string;
}

export interface HostRelation {
  id: string;
  name: string;
  host_type: string;
  hostname: string | null;
  ip_address: string | null;
  status: string;
  role: string;
  relation_notes: string | null;
}

export interface HostWithRelations extends Host {
  applications: ApplicationHostRelation[];
}

export interface ApplicationHostRelation {
  id: string;
  name: string;
  status: string;
  role: string;
}

export interface LinkHost {
  role?: string;
  notes?: string;
}

// Domain types
export interface Domain {
  id: string;
  name: string;
  registrar: string | null;
  dns_provider: string | null;
  expires_at: string | null;
  ssl_expires_at: string | null;
  ssl_issuer: string | null;
  status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateDomain {
  name: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  ssl_expires_at?: string;
  ssl_issuer?: string;
  status?: string;
  notes?: string;
}

export interface UpdateDomain {
  name?: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  ssl_expires_at?: string;
  ssl_issuer?: string;
  status?: string;
  notes?: string;
}

export interface DomainRelation {
  id: string;
  name: string;
  registrar: string | null;
  expires_at: string | null;
  ssl_expires_at: string | null;
  status: string;
  record_type: string;
  target: string | null;
  target_host_id: string | null;
  target_host_name: string | null;
  is_primary: boolean;
  relation_notes: string | null;
}

export interface DomainWithRelations extends Domain {
  applications: ApplicationDomainRelation[];
}

export interface ApplicationDomainRelation {
  id: string;
  name: string;
  status: string;
  record_type: string;
  is_primary: boolean;
}

export interface LinkDomain {
  record_type?: string;
  target?: string | null;
  target_host_id?: string | null;
  is_primary?: boolean;
  notes?: string;
}

// Person types
export interface Person {
  id: string;
  name: string;
  email: string | null;
  role: string | null;
  department: string | null;
  phone: string | null;
  is_active: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreatePerson {
  name: string;
  email?: string;
  role?: string;
  department?: string;
  phone?: string;
  is_active?: boolean;
  notes?: string;
}

export interface UpdatePerson {
  name?: string;
  email?: string;
  role?: string;
  department?: string;
  phone?: string;
  is_active?: boolean;
  notes?: string;
}

export interface PersonRelation {
  id: string;
  name: string;
  email: string | null;
  role: string | null;
  is_active: boolean;
  contribution_type: string;
  start_date: string | null;
  end_date: string | null;
  relation_notes: string | null;
}

export interface PersonWithRelations extends Person {
  applications: ApplicationPersonRelation[];
}

export interface ApplicationPersonRelation {
  id: string;
  name: string;
  status: string;
  contribution_type: string;
}

export interface LinkPerson {
  contribution_type?: string;
  start_date?: string;
  end_date?: string;
  notes?: string;
}

// Network Share types
export interface NetworkShare {
  id: string;
  name: string;
  path: string;
  share_type: string;
  server: string | null;
  purpose: string | null;
  status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateNetworkShare {
  name: string;
  path: string;
  share_type?: string;
  server?: string;
  purpose?: string;
  status?: string;
  notes?: string;
}

export interface UpdateNetworkShare {
  name?: string;
  path?: string;
  share_type?: string;
  server?: string;
  purpose?: string;
  status?: string;
  notes?: string;
}

export interface NetworkShareRelation {
  id: string;
  name: string;
  path: string;
  share_type: string;
  server: string | null;
  status: string;
  usage: string | null;
  mount_point: string | null;
  permissions: string | null;
  relation_notes: string | null;
}

export interface NetworkShareWithRelations extends NetworkShare {
  applications: ApplicationNetworkShareRelation[];
}

export interface ApplicationNetworkShareRelation {
  id: string;
  name: string;
  status: string;
  usage: string | null;
  mount_point: string | null;
}

export interface LinkNetworkShare {
  usage?: string;
  mount_point?: string;
  permissions?: string;
  notes?: string;
}

// Note types
export interface Note {
  id: string;
  entity_type: string;
  entity_id: string;
  title: string;
  content: string | null;
  note_type: string;
  url: string | null;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateNote {
  entity_type: string;
  entity_id: string;
  title: string;
  content?: string;
  note_type?: string;
  url?: string;
  is_pinned?: boolean;
}

export interface UpdateNote {
  title?: string;
  content?: string;
  note_type?: string;
  url?: string;
  is_pinned?: boolean;
}

// Dashboard types
export interface DashboardStats {
  applications: EntityStats;
  hosts: EntityStats;
  domains: EntityStats;
  people: EntityStats;
  network_shares: EntityStats;
  notes: number;
  expiring_domains: ExpiringDomain[];
  expiring_ssl: ExpiringSsl[];
}

export interface EntityStats {
  total: number;
  active: number;
}

export interface ExpiringDomain {
  id: string;
  name: string;
  expires_at: string | null;
}

export interface ExpiringSsl {
  id: string;
  name: string;
  ssl_expires_at: string | null;
}

// Search types
export interface SearchResults {
  applications: SearchResult[];
  hosts: SearchResult[];
  domains: SearchResult[];
  people: SearchResult[];
  network_shares: SearchResult[];
}

export interface SearchResult {
  id: string;
  name: string;
  description: string | null;
  entity_type: string;
}

// Stack types
export interface Stack {
  id: string;
  name: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateStack {
  name: string;
  notes?: string;
}

export interface UpdateStack {
  name?: string;
  notes?: string;
}

export interface StackRelation {
  id: string;
  name: string;
}

export interface StackWithRelations extends Stack {
  applications: ApplicationStackRelation[];
}

export interface ApplicationStackRelation {
  id: string;
  name: string;
  status: string;
}
