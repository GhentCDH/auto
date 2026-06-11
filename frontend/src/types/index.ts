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

// Filter params for each entity type
export interface ApplicationFilterParams extends PaginationParams {
  status?: string;
  environment?: string;
}

export interface ServiceFilterParams extends PaginationParams {
  status?: string;
  environment?: string;
}

export interface PersonFilterParams extends PaginationParams {
  is_active?: boolean;
}

export interface ShareFilterParams extends PaginationParams {
  status?: string;
  share_type?: string;
}

export interface InfraFilterParams extends PaginationParams {
  type?: string;
  ip?: string;
}

export interface DomainFilterParams extends PaginationParams {}

// Image reference types
export interface ImageRef {
  url: string;
  alias?: string;
}

// Application types
export interface Application {
  id: string;
  name: string;
  description: string | null;
  repository_url: string | null;
  environment: string;
  url: string | null;
  status: string;
  image_refs: string | null;
  outline_url: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateApplication {
  name: string;
  description?: string;
  repository_url?: string;
  environment?: string;
  url?: string;
  status?: string;
  image_refs?: string;
  outline_url?: string;
}

export interface UpdateApplication {
  name?: string;
  description?: string;
  repository_url?: string;
  environment?: string;
  url?: string;
  status?: string;
  image_refs?: string;
  outline_url?: string;
}

export interface ApplicationWithRelations extends Application {
  infra: InfraRelation[];
  services: ServiceRelation[];
  domains: DomainRelation[];
  people: PersonRelation[];
  network_shares: NetworkShareRelation[];
  notes: Note[];
  stacks: StackRelation[];
  healthchecks: HealthcheckRelation[];
}

// Service types
export interface Service {
  id: string;
  name: string;
  description: string | null;
  repository_url: string | null;
  environment: string;
  status: string;
  image_refs: string | null;
  outline_url: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateService {
  name: string;
  description?: string;
  repository_url?: string;
  environment?: string;
  status?: string;
  image_refs?: string;
  outline_url?: string;
}

export interface UpdateService {
  name?: string;
  description?: string;
  repository_url?: string;
  environment?: string;
  status?: string;
  image_refs?: string;
  outline_url?: string;
}

export interface ServiceRelation {
  id: string;
  name: string;
  environment: string;
  status: string;
  relation_notes: string | null;
}

export interface ServiceWithRelations extends Service {
  applications: ApplicationServiceRelation[];
  infra: InfraRelation[];
  healthchecks: HealthcheckRelation[];
}

export interface ApplicationServiceRelation {
  id: string;
  name: string;
  environment: string;
  status: string;
}

export interface LinkService {
  notes?: string;
}

// Infra types
export interface Infra {
  id: string;
  name: string;
  description: string | null;
  type: string;
  created_at: string;
  updated_at: string;
  created_by: string | null;
  /** Present on list endpoint responses; absent on lightweight relation payloads. */
  ips?: InfraIp[];
  /** Kuma monitor IDs for enabled healthchecks on linked apps/services (list endpoint). */
  healthcheck_kuma_ids?: number[];
}

/** A new domain created alongside an infra. No target — the server points it
 *  at the infra being created, breaking the otherwise-circular create order. */
export interface NewInfraDomain {
  fqdn: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  notes?: string;
}

export interface CreateInfra {
  name: string;
  description?: string;
  type: string;
  /** If set, points this existing domain at the infra so its IPs resolve from DNS. */
  domain_id?: string;
  /** Create a fresh domain targeting this infra in the same request. */
  new_domain?: NewInfraDomain;
  /** Manually-assigned IPs (used when the infra has no domain). */
  manual_ips?: string[];
}

export interface UpdateInfra {
  name?: string;
  description?: string;
  type?: string;
  domain_id?: string;
  new_domain?: NewInfraDomain;
  manual_ips?: string[];
}

export interface InfraIp {
  ip: string;
  source: string; // 'domain' | 'manual'
  last_synced_at: string;
}

export interface InfraRelation {
  id: string;
  name: string;
  type: string;
  relation_notes: string | null;
}

export interface InfraWithRelations extends Infra {
  ips: InfraIp[];
  domain: { id: string; fqdn: string } | null;
  applications: ApplicationInfraRelation[];
  services: ServiceInfraRelation[];
  healthchecks: InfraHealthcheckRelation[];
}

export interface ApplicationInfraRelation {
  id: string;
  name: string;
  environment: string;
  status: string;
}

export interface ServiceInfraRelation {
  id: string;
  name: string;
  environment: string;
  status: string;
}

export interface LinkInfra {
  notes?: string;
}

// Domain types
export interface Domain {
  id: string;
  fqdn: string;
  registrar: string | null;
  dns_provider: string | null;
  expires_at: string | null;
  notes: string | null;
  target_application_id: string | null;
  target_application_name: string | null;
  target_service_id: string | null;
  target_service_name: string | null;
  target_infra_id: string | null;
  target_infra_name: string | null;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface DomainNamed extends Domain {
  name: string;
}

/** A backlink from a DNS A/AAAA record to a known infra (matched by IP). */
export interface InfraMatch {
  id: string;
  name: string;
}

/** A single DNS record resolved live for a domain's FQDN. */
export interface DnsRecord {
  record_type: string;
  value: string;
  ttl: number;
  priority: number | null;
  /** Set when this record's IP matches a tracked infra. */
  infra?: InfraMatch;
}

/** Result of a live DNS lookup for one FQDN (records are not persisted). */
export interface DnsLookup {
  fqdn: string;
  records: DnsRecord[];
  resolved_at: string;
  error?: string;
}

export interface CreateDomain {
  fqdn: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  target_application_id?: string;
  target_service_id?: string;
  target_infra_id?: string;
  notes?: string;
}

export interface UpdateDomain {
  fqdn?: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  target_application_id?: string;
  target_service_id?: string;
  target_infra_id?: string;
  status?: string;
  notes?: string;
}

export interface DomainRelation {
  id: string;
  fqdn: string;
  target_application_id: string | null;
  target_application_name: string | null;
  target_service_id: string | null;
  target_service_name: string | null;
  relation_notes: string | null;
}

export interface DomainWithRelations extends Domain {
  applications: ApplicationDomainRelation[];
}

export interface DomainNamedWithRelations extends DomainWithRelations {
  name: string;
}

export interface ApplicationDomainRelation {
  id: string;
  name: string;
}

export interface LinkDomain {
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
  services: EntityStats;
  infra: EntityStats;
  domains: EntityStats;
  people: EntityStats;
  network_shares: EntityStats;
  notes: number;
  expiring_domains: ExpiringDomain[];
  healthchecks: HealthcheckStats;
  recent_activity: RecentActivity[];
}

export interface HealthcheckStats {
  total: number;
  enabled: number;
  kuma_dirty: number;
}

export interface RecentActivity {
  id: string;
  name: string;
  entity_type: string;
  updated_at: string;
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

// Search types
export interface SearchResults {
  applications: SearchResult[];
  services: SearchResult[];
  infra: SearchResult[];
  domains: SearchResult[];
  people: SearchResult[];
  network_shares: SearchResult[];
  stacks: SearchResult[];
  healthchecks: SearchResult[];
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

// Healthcheck types
export interface Healthcheck {
  id: string;
  name: string;
  application_id: string | null;
  service_id: string | null;
  kuma_id: number | null;
  domain_id: string;
  protocol: string;
  path: string;
  method: string;
  headers: string | null;
  expected_status: number;
  expected_body: string | null;
  timeout_seconds: number;
  interval: number;
  is_enabled: boolean;
  notes: string | null;
  retry: number;
  retry_interval: number;
  request_body_encoding: string;
  request_body: string | null;
  http_auth_user: string | null;
  http_auth_pass: string | null;
  notifications: boolean;
  kuma_dirty: boolean;
  created_at: string;
  updated_at: string;
  created_by: string | null;
}

export interface CreateHealthcheck {
  name: string;
  application_id?: string;
  service_id?: string;
  kuma_id?: number;
  domain_id: string;
  protocol?: string;
  path?: string;
  method?: string;
  headers?: string;
  expected_status?: number;
  expected_body?: string;
  timeout_seconds?: number;
  interval?: number;
  is_enabled?: boolean;
  notes?: string;
  retry?: number;
  retry_interval?: number;
  request_body_encoding?: string;
  request_body?: string;
  http_auth_user?: string;
  http_auth_pass?: string;
  notifications?: boolean;
}

export interface UpdateHealthcheck {
  name?: string;
  application_id?: string;
  service_id?: string;
  kuma_id?: number;
  domain_id?: string;
  protocol?: string;
  path?: string;
  method?: string;
  headers?: string;
  expected_status?: number;
  expected_body?: string;
  timeout_seconds?: number;
  interval?: number;
  is_enabled?: boolean;
  notes?: string;
  retry?: number;
  retry_interval?: number;
  request_body_encoding?: string;
  request_body?: string;
  http_auth_user?: string;
  http_auth_pass?: string;
  notifications?: boolean;
}

export interface HealthcheckWithRelations extends Healthcheck {
  application_name: string | null;
  service_name: string | null;
  domain_fqdn: string;
  parsed_headers: Record<string, string> | null;
}

export interface HealthcheckRelation {
  id: string;
  name: string;
  protocol: string;
  domain_fqdn: string;
  path: string;
  expected_status: number;
  is_enabled: boolean;
  notifications: boolean;
  kuma_id: number | null;
  kuma_dirty: boolean;
}

export interface InfraHealthcheckRelation extends HealthcheckRelation {
  parent_type: 'application' | 'service';
  parent_id: string;
  parent_name: string;
}

export interface HealthcheckExecuteResult {
  healthcheck_id: string;
  url: string;
  success: boolean;
  status_code: number | null;
  response_time_ms: number;
  body_match: boolean | null;
  error: string | null;
  executed_at: string;
}

export interface HealthcheckFilterParams extends PaginationParams {
  application_id?: string;
  service_id?: string;
  is_enabled?: boolean;
}

export interface KumaMonitor {
  name: string;
  url: string;
  method: string;
  expected_status: number;
  timeout: number;
  headers: Record<string, string> | null;
  target_type: string;
  target_name: string;
}

export interface KumaEndpoint {
  url: string;
}

// Kuma import types (for importing from Uptime Kuma export)
export interface KumaMonitorImport {
  kuma_id: number;
  name: string;
  url: string;
  hostname: string;
  path: string;
  protocol: string;
  method: string;
  expected_status: number;
  timeout_seconds: number;
  interval: number;
  headers: string | null;
  keyword: string | null;
  // New fields for retry and body
  retry: number;
  retry_interval: number;
  request_body: string | null;
  request_body_encoding: string;
  http_auth_user: string | null;
  http_auth_pass: string | null;
}

export interface ImportMapping {
  monitor: KumaMonitorImport;
  domain_id: string | null; // null = needs creation
  domain_fqdn: string; // for display/creation
  // Healthcheck target
  application_id: string | null;
  service_id: string | null;
  target_name: string; // for display
  // Domain target (used when creating new domain)
  domain_application_id: string | null;
  domain_service_id: string | null;
  domain_target_name: string;
  skip: boolean;
}

// --- Server-provided configuration (GET /api/config) ---
// Mirrors the backend `PublicConfig`: form defaults + dropdown option lists.

export interface ApplicationDefaults {
  environment: string;
  status: string;
}

export interface ServiceDefaults {
  environment: string;
  status: string;
}

export interface HealthcheckDefaults {
  protocol: string;
  path: string;
  method: string;
  expected_status: number;
  timeout_seconds: number;
  interval: number;
  is_enabled: boolean;
  notifications: boolean;
  retry: number;
  retry_interval: number;
  request_body_encoding: string;
}

export interface NoteDefaults {
  note_type: string;
}

export interface PersonDefaults {
  department: string;
  email_domain: string;
  contribution_type: string;
  is_active: boolean;
}

export interface ShareDefaults {
  path_prefix: string;
  server: string;
  share_type: string;
  status: string;
  usage: string;
  permissions: string;
}

export interface InfraDefaults {
  vm_name_prefix: string;
}

export interface DomainDefaults {
  target_type: string;
}

export interface Defaults {
  application: ApplicationDefaults;
  service: ServiceDefaults;
  healthcheck: HealthcheckDefaults;
  note: NoteDefaults;
  person: PersonDefaults;
  share: ShareDefaults;
  infra: InfraDefaults;
  domain: DomainDefaults;
}

// Ordered value -> label maps for dropdowns.
export interface Options {
  statuses: Record<string, string>;
  environments: Record<string, string>;
  infra_types: Record<string, string>;
  share_usages: Record<string, string>;
  share_types: Record<string, string>;
  domain_types: Record<string, string>;
  domain_status: Record<string, string>;
  contribution_types: Record<string, string>;
  note_types: Record<string, string>;
}

export interface PublicConfig {
  defaults: Defaults;
  options: Options;
}
