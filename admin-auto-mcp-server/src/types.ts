/**
 * TypeScript type definitions for Admin Auto API
 */

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
  [key: string]: unknown;
}

export interface Application {
  id: string;
  name: string;
  description?: string;
  repository_url?: string;
  environment: string;
  url?: string;
  status: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
  [key: string]: unknown;
}

export interface ApplicationDetail extends Application {
  infra: Infrastructure[];
  services: Service[];
  domains: Domain[];
  people: Person[];
  network_shares: NetworkShare[];
  notes: Note[];
  stacks: Stack[];
  healthchecks: Healthcheck[];
  [key: string]: unknown;
}

export interface Service {
  id: string;
  name: string;
  description?: string;
  repository_url?: string;
  environment: string;
  status: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface ServiceDetail extends Service {
  applications: Application[];
  infra: Infrastructure[];
  healthchecks: Healthcheck[];
}

export interface Infrastructure {
  id: string;
  name: string;
  description?: string;
  type: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface InfrastructureDetail extends Infrastructure {
  applications: Application[];
  services: Service[];
}

export interface Domain {
  id: string;
  fqdn: string;
  registrar?: string;
  dns_provider?: string;
  expires_at?: string;
  notes?: string;
  target_application_id?: string;
  target_service_id?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface DomainDetail extends Domain {
  applications: Application[];
  target_application?: Application;
  target_service?: Service;
}

export interface Person {
  id: string;
  name: string;
  email?: string;
  role?: string;
  department?: string;
  phone?: string;
  is_active: boolean;
  notes?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface PersonDetail extends Person {
  applications: Array<{
    application: Application;
    contribution_type: string;
    start_date?: string;
    end_date?: string;
    notes?: string;
  }>;
}

export interface NetworkShare {
  id: string;
  name: string;
  path: string;
  share_type: string;
  server?: string;
  purpose?: string;
  status: string;
  notes?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface NetworkShareDetail extends NetworkShare {
  applications: Application[];
}

export interface Note {
  id: string;
  entity_type: string;
  entity_id: string;
  title: string;
  content?: string;
  note_type: string;
  url?: string;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface Stack {
  id: string;
  name: string;
  notes?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface StackDetail extends Stack {
  applications: Application[];
}

export interface Healthcheck {
  id: string;
  name: string;
  application_id?: string;
  service_id?: string;
  domain_id: string;
  protocol: string;
  path: string;
  method: string;
  headers?: string;
  expected_status: number;
  expected_body?: string;
  timeout_seconds: number;
  is_enabled: boolean;
  notes?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
}

export interface HealthcheckDetail extends Healthcheck {
  application_name?: string;
  service_name?: string;
  domain_fqdn?: string;
  parsed_headers?: Record<string, string>;
}

export interface HealthcheckResult {
  healthcheck_id: string;
  url: string;
  success: boolean;
  status_code?: number;
  response_time_ms?: number;
  body_match?: boolean;
  error?: string;
  executed_at: string;
}

export interface DashboardStats {
  applications_count: number;
  services_count: number;
  infrastructure_count: number;
  domains_count: number;
  people_count: number;
  healthchecks_count: number;
  // Add other stats fields as needed
}

export interface SearchResults {
  applications: Application[];
  services: Service[];
  infrastructure: Infrastructure[];
  domains: Domain[];
  people: Person[];
  network_shares: NetworkShare[];
}
