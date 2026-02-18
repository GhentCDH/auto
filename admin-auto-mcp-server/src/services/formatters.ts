/**
 * Response formatters for converting API data to Markdown or JSON
 */

import { ResponseFormat, CHARACTER_LIMIT } from "../constants.js";
import type {
  Application,
  ApplicationDetail,
  Service,
  ServiceDetail,
  Infrastructure,
  InfrastructureDetail,
  Domain,
  DomainDetail,
  Person,
  PersonDetail,
  NetworkShare,
  NetworkShareDetail,
  Note,
  Stack,
  StackDetail,
  Healthcheck,
  HealthcheckDetail,
  PaginatedResponse
} from "../types.js";

/**
 * Format a date string to human-readable format
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
}

/**
 * Truncate content if it exceeds CHARACTER_LIMIT
 */
export function truncateIfNeeded(content: string, metadata?: any): { content: string; truncated: boolean } {
  if (content.length <= CHARACTER_LIMIT) {
    return { content, truncated: false };
  }

  const truncatedContent = content.substring(0, CHARACTER_LIMIT);
  const lastNewline = truncatedContent.lastIndexOf("\n");
  const finalContent = lastNewline > CHARACTER_LIMIT * 0.9
    ? truncatedContent.substring(0, lastNewline)
    : truncatedContent;

  const truncationMessage = metadata
    ? `\n\n[TRUNCATED: Content exceeded ${CHARACTER_LIMIT} characters. Use pagination or filters to see more results.]`
    : `\n\n[TRUNCATED: Content exceeded ${CHARACTER_LIMIT} characters.]`;

  return {
    content: finalContent + truncationMessage,
    truncated: true
  };
}

/**
 * Format paginated list response
 */
export function formatPaginatedList<T>(
  data: PaginatedResponse<T>,
  itemFormatter: (item: T) => string,
  format: ResponseFormat
): string {
  if (format === ResponseFormat.JSON) {
    return JSON.stringify(data, null, 2);
  }

  const lines: string[] = [];
  lines.push(`# Results (Page ${data.page} of ${data.total_pages})`);
  lines.push("");
  lines.push(`**Total:** ${data.total} items | **Showing:** ${data.data.length} items`);
  lines.push("");

  if (data.data.length === 0) {
    lines.push("*No results found*");
  } else {
    data.data.forEach((item, index) => {
      lines.push(itemFormatter(item));
      if (index < data.data.length - 1) {
        lines.push("");
      }
    });
  }

  if (data.page < data.total_pages) {
    lines.push("");
    lines.push(`*Use page=${data.page + 1} to see the next page*`);
  }

  return lines.join("\n");
}

/**
 * Format Application list item
 */
export function formatApplicationListItem(app: Application): string {
  const lines = [
    `## ${app.name} (${app.id})`,
    `- **Environment:** ${app.environment}`,
    `- **Status:** ${app.status}`
  ];
  if (app.description) lines.push(`- **Description:** ${app.description}`);
  if (app.repository_url) lines.push(`- **Repository:** ${app.repository_url}`);
  if (app.url) lines.push(`- **URL:** ${app.url}`);
  lines.push(`- **Updated:** ${formatDate(app.updated_at)}`);
  return lines.join("\n");
}

/**
 * Format Application detail
 */
export function formatApplicationDetail(app: ApplicationDetail, format: ResponseFormat): string {
  if (format === ResponseFormat.JSON) {
    return JSON.stringify(app, null, 2);
  }

  const lines = [
    `# ${app.name}`,
    "",
    `**ID:** ${app.id}`,
    `**Environment:** ${app.environment}`,
    `**Status:** ${app.status}`
  ];

  if (app.description) lines.push(`**Description:** ${app.description}`);
  if (app.repository_url) lines.push(`**Repository:** ${app.repository_url}`);
  if (app.url) lines.push(`**URL:** ${app.url}`);

  lines.push("");
  lines.push(`**Created:** ${formatDate(app.created_at)}`);
  lines.push(`**Updated:** ${formatDate(app.updated_at)}`);
  if (app.created_by) lines.push(`**Created By:** ${app.created_by}`);

  // Related entities
  if (app.infra && app.infra.length > 0) {
    lines.push("");
    lines.push("## Infrastructure");
    app.infra.forEach(i => {
      lines.push(`- ${i.name} (${i.type}) - ${i.id}`);
    });
  }

  if (app.services && app.services.length > 0) {
    lines.push("");
    lines.push("## Services");
    app.services.forEach(s => {
      lines.push(`- ${s.name} (${s.environment}) - ${s.id}`);
    });
  }

  if (app.domains && app.domains.length > 0) {
    lines.push("");
    lines.push("## Domains");
    app.domains.forEach(d => {
      lines.push(`- ${d.fqdn} - ${d.id}`);
    });
  }

  if (app.people && app.people.length > 0) {
    lines.push("");
    lines.push("## People");
    app.people.forEach(p => {
      lines.push(`- ${p.name} (${p.role || 'N/A'}) - ${p.id}`);
    });
  }

  if (app.network_shares && app.network_shares.length > 0) {
    lines.push("");
    lines.push("## Network Shares");
    app.network_shares.forEach(s => {
      lines.push(`- ${s.name} (${s.share_type}) - ${s.path}`);
    });
  }

  if (app.stacks && app.stacks.length > 0) {
    lines.push("");
    lines.push("## Technology Stacks");
    app.stacks.forEach(s => {
      lines.push(`- ${s.name} - ${s.id}`);
    });
  }

  if (app.healthchecks && app.healthchecks.length > 0) {
    lines.push("");
    lines.push("## Healthchecks");
    app.healthchecks.forEach(h => {
      lines.push(`- ${h.name} - ${h.protocol}://${h.path} (${h.is_enabled ? 'enabled' : 'disabled'})`);
    });
  }

  if (app.notes && app.notes.length > 0) {
    lines.push("");
    lines.push("## Notes");
    app.notes.forEach(n => {
      lines.push(`- **${n.title}** (${n.note_type})${n.is_pinned ? ' 📌' : ''}`);
      if (n.content) lines.push(`  ${n.content.substring(0, 100)}${n.content.length > 100 ? '...' : ''}`);
    });
  }

  return lines.join("\n");
}

/**
 * Format Service list item
 */
export function formatServiceListItem(service: Service): string {
  const lines = [
    `## ${service.name} (${service.id})`,
    `- **Environment:** ${service.environment}`,
    `- **Status:** ${service.status}`
  ];
  if (service.description) lines.push(`- **Description:** ${service.description}`);
  if (service.repository_url) lines.push(`- **Repository:** ${service.repository_url}`);
  lines.push(`- **Updated:** ${formatDate(service.updated_at)}`);
  return lines.join("\n");
}

/**
 * Format Infrastructure list item
 */
export function formatInfrastructureListItem(infra: Infrastructure): string {
  const lines = [
    `## ${infra.name} (${infra.id})`,
    `- **Type:** ${infra.type}`
  ];
  if (infra.description) lines.push(`- **Description:** ${infra.description}`);
  lines.push(`- **Updated:** ${formatDate(infra.updated_at)}`);
  return lines.join("\n");
}

/**
 * Format Domain list item
 */
export function formatDomainListItem(domain: Domain): string {
  const lines = [
    `## ${domain.fqdn} (${domain.id})`
  ];
  if (domain.registrar) lines.push(`- **Registrar:** ${domain.registrar}`);
  if (domain.dns_provider) lines.push(`- **DNS Provider:** ${domain.dns_provider}`);
  if (domain.expires_at) lines.push(`- **Expires:** ${formatDate(domain.expires_at)}`);
  lines.push(`- **Updated:** ${formatDate(domain.updated_at)}`);
  return lines.join("\n");
}

/**
 * Format Person list item
 */
export function formatPersonListItem(person: Person): string {
  const lines = [
    `## ${person.name} (${person.id})`,
    `- **Active:** ${person.is_active ? 'Yes' : 'No'}`
  ];
  if (person.email) lines.push(`- **Email:** ${person.email}`);
  if (person.role) lines.push(`- **Role:** ${person.role}`);
  if (person.department) lines.push(`- **Department:** ${person.department}`);
  return lines.join("\n");
}

/**
 * Format Network Share list item
 */
export function formatNetworkShareListItem(share: NetworkShare): string {
  const lines = [
    `## ${share.name} (${share.id})`,
    `- **Type:** ${share.share_type}`,
    `- **Path:** ${share.path}`,
    `- **Status:** ${share.status}`
  ];
  if (share.server) lines.push(`- **Server:** ${share.server}`);
  if (share.purpose) lines.push(`- **Purpose:** ${share.purpose}`);
  return lines.join("\n");
}

/**
 * Format Healthcheck list item
 */
export function formatHealthcheckListItem(healthcheck: Healthcheck): string {
  const lines = [
    `## ${healthcheck.name} (${healthcheck.id})`,
    `- **URL:** ${healthcheck.protocol}://${healthcheck.path}`,
    `- **Method:** ${healthcheck.method}`,
    `- **Expected Status:** ${healthcheck.expected_status}`,
    `- **Enabled:** ${healthcheck.is_enabled ? 'Yes' : 'No'}`
  ];
  return lines.join("\n");
}
