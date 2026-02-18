#!/usr/bin/env node
/**
 * MCP Server for Admin Auto API
 *
 * Provides tools to interact with an Admin Auto instance for infrastructure management,
 * including applications, services, infrastructure, domains, people, healthchecks, and more.
 */

// Load environment variables from .env file
import dotenv from "dotenv";
dotenv.config();

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import express from "express";
import { z } from "zod";
import { AutoApiClient } from "./services/api-client.js";
import { ResponseFormat } from "./constants.js";
import {
  formatPaginatedList,
  formatApplicationListItem,
  formatApplicationDetail,
  formatServiceListItem,
  formatInfrastructureListItem,
  formatDomainListItem,
  formatPersonListItem,
  formatNetworkShareListItem,
  formatHealthcheckListItem,
  truncateIfNeeded
} from "./services/formatters.js";
import type {
  PaginatedResponse,
  Application,
  ApplicationDetail,
  Service,
  Infrastructure,
  Domain,
  Person,
  NetworkShare,
  Healthcheck,
  SearchResults,
  DashboardStats
} from "./types.js";

// Environment validation
const AUTO_URL = process.env.AUTO_URL;
const AUTO_USERNAME = process.env.AUTO_USERNAME;
const AUTO_PASSWORD = process.env.AUTO_PASSWORD;

if (!AUTO_URL || !AUTO_USERNAME || !AUTO_PASSWORD) {
  console.error("ERROR: Missing required environment variables:");
  console.error("  - AUTO_URL: Base URL of the Admin Auto instance (e.g., https://auto.ghentcdh.be)");
  console.error("  - AUTO_USERNAME: HTTP Basic Auth username");
  console.error("  - AUTO_PASSWORD: HTTP Basic Auth password");
  process.exit(1);
}

// Create API client
const apiClient = new AutoApiClient(AUTO_URL, AUTO_USERNAME, AUTO_PASSWORD);

// Create MCP server
const server = new McpServer({
  name: "admin-auto-mcp-server",
  version: "1.0.0"
});

// ============================================================================
// SEARCH AND DASHBOARD TOOLS
// ============================================================================

server.registerTool(
  "auto_global_search",
  {
    title: "Global Search Across All Resources",
    description: `Search across all resource types in the Admin Auto system (applications, services, infrastructure, domains, people, network shares).

This is the most efficient tool for finding resources when you don't know the exact resource type or ID. It searches by name, description, and other text fields across all entities.

Args:
  - query (string): Search query string (minimum 1 character)
  - response_format ('markdown' | 'json'): Output format (default: 'markdown')

Returns:
  Search results grouped by resource type, showing matching applications, services, infrastructure, domains, people, and network shares.

Examples:
  - Use when: "Find anything related to GitLab" -> params with query="gitlab"
  - Use when: "Search for docker resources" -> params with query="docker"
  - Use when: "Find production kubernetes" -> params with query="production kubernetes"`,
    inputSchema: z.object({
      query: z.string().min(1).describe("Search query string"),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const results = await apiClient.get<SearchResults>("/search", { q: params.query });

      if (params.response_format === ResponseFormat.JSON) {
        const output = { query: params.query, results };
        const { content, truncated } = truncateIfNeeded(JSON.stringify(output, null, 2), output);
        return {
          content: [{ type: "text", text: content }],
          structuredContent: output
        };
      }

      // Markdown format
      const lines: string[] = [`# Search Results for "${params.query}"`, ""];

      let totalResults = 0;

      if (results.applications && results.applications.length > 0) {
        totalResults += results.applications.length;
        lines.push(`## Applications (${results.applications.length})`);
        results.applications.forEach(app => {
          lines.push(`- **${app.name}** (${app.environment}) - ${app.id}`);
          if (app.description) lines.push(`  ${app.description.substring(0, 100)}`);
        });
        lines.push("");
      }

      if (results.services && results.services.length > 0) {
        totalResults += results.services.length;
        lines.push(`## Services (${results.services.length})`);
        results.services.forEach(svc => {
          lines.push(`- **${svc.name}** (${svc.environment}) - ${svc.id}`);
          if (svc.description) lines.push(`  ${svc.description.substring(0, 100)}`);
        });
        lines.push("");
      }

      if (results.infrastructure && results.infrastructure.length > 0) {
        totalResults += results.infrastructure.length;
        lines.push(`## Infrastructure (${results.infrastructure.length})`);
        results.infrastructure.forEach(infra => {
          lines.push(`- **${infra.name}** (${infra.type}) - ${infra.id}`);
        });
        lines.push("");
      }

      if (results.domains && results.domains.length > 0) {
        totalResults += results.domains.length;
        lines.push(`## Domains (${results.domains.length})`);
        results.domains.forEach(domain => {
          lines.push(`- **${domain.fqdn}** - ${domain.id}`);
        });
        lines.push("");
      }

      if (results.people && results.people.length > 0) {
        totalResults += results.people.length;
        lines.push(`## People (${results.people.length})`);
        results.people.forEach(person => {
          lines.push(`- **${person.name}** (${person.role || 'N/A'}) - ${person.id}`);
        });
        lines.push("");
      }

      if (results.network_shares && results.network_shares.length > 0) {
        totalResults += results.network_shares.length;
        lines.push(`## Network Shares (${results.network_shares.length})`);
        results.network_shares.forEach(share => {
          lines.push(`- **${share.name}** (${share.share_type}) - ${share.path}`);
        });
        lines.push("");
      }

      if (totalResults === 0) {
        lines.push("*No results found*");
      } else {
        lines.unshift(`**Total Results:** ${totalResults}`, "");
      }

      const markdown = lines.join("\n");
      const { content } = truncateIfNeeded(markdown);
      return {
        content: [{ type: "text", text: content }]
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_get_dashboard_stats",
  {
    title: "Get Dashboard Statistics",
    description: `Get aggregated statistics and counts for the Admin Auto dashboard.

Returns summary counts for all major resource types in the system.

Returns:
  Dashboard statistics including counts for applications, services, infrastructure, domains, people, healthchecks, and other resources.`,
    inputSchema: z.object({}).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async () => {
    try {
      const stats = await apiClient.get<DashboardStats>("/dashboard/stats");
      const output = { stats };
      return {
        content: [{ type: "text", text: JSON.stringify(stats, null, 2) }],
        structuredContent: output
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

// ============================================================================
// APPLICATION TOOLS
// ============================================================================

server.registerTool(
  "auto_list_applications",
  {
    title: "List Applications",
    description: `List all applications with pagination and optional filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by name/description
  - environment ('prd'|'dev'|'stg'|'tst'): Filter by environment
  - status ('active'|'inactive'|'deprecated'): Filter by status
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of applications with their basic information.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      environment: z.enum(["prd", "dev", "stg", "tst"]).optional(),
      status: z.enum(["active", "inactive", "deprecated"]).optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;
      if (params.environment) queryParams.environment = params.environment;
      if (params.status) queryParams.status = params.status;

      const data = await apiClient.get<PaginatedResponse<Application>>("/applications", queryParams);

      const formatted = formatPaginatedList(data, formatApplicationListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_get_application",
  {
    title: "Get Application Details",
    description: `Get detailed information about a specific application including all related resources.

Args:
  - id (string): Application ID (UUID format)
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Complete application details including related infrastructure, services, domains, people, network shares, stacks, healthchecks, and notes.`,
    inputSchema: z.object({
      id: z.string().uuid(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const app = await apiClient.get<ApplicationDetail>(`/applications/${params.id}`);
      const formatted = formatApplicationDetail(app, params.response_format);
      const { content } = truncateIfNeeded(formatted, app);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: app })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_create_application",
  {
    title: "Create Application",
    description: `Create a new application in the Admin Auto system.

Args:
  - name (string): Application name (required)
  - description (string): Application description
  - repository_url (string): Git repository URL
  - environment ('prd'|'dev'|'stg'|'tst'): Environment, default 'prd'
  - url (string): Application URL
  - status ('active'|'inactive'|'deprecated'): Status, default 'active'

Returns:
  The newly created application object with its generated ID.`,
    inputSchema: z.object({
      name: z.string().min(1).max(255),
      description: z.string().optional(),
      repository_url: z.string().url().optional(),
      environment: z.enum(["prd", "dev", "stg", "tst"]).default("prd"),
      url: z.string().url().optional(),
      status: z.enum(["active", "inactive", "deprecated"]).default("active")
    }).strict(),
    annotations: {
      readOnlyHint: false,
      destructiveHint: false,
      idempotentHint: false,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const app = await apiClient.post<Application>("/applications", params);
      return {
        content: [{ type: "text", text: `Successfully created application "${app.name}" with ID: ${app.id}` }],
        structuredContent: app
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

// ============================================================================
// SERVICE TOOLS
// ============================================================================

server.registerTool(
  "auto_list_services",
  {
    title: "List Services",
    description: `List all services with pagination and optional filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by name/description
  - environment ('prd'|'dev'|'stg'|'tst'): Filter by environment
  - status ('active'|'inactive'|'deprecated'): Filter by status
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of services with their basic information.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      environment: z.enum(["prd", "dev", "stg", "tst"]).optional(),
      status: z.enum(["active", "inactive", "deprecated"]).optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;
      if (params.environment) queryParams.environment = params.environment;
      if (params.status) queryParams.status = params.status;

      const data = await apiClient.get<PaginatedResponse<Service>>("/services", queryParams);

      const formatted = formatPaginatedList(data, formatServiceListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

// ============================================================================
// INFRASTRUCTURE TOOLS
// ============================================================================

server.registerTool(
  "auto_list_infrastructure",
  {
    title: "List Infrastructure Resources",
    description: `List all infrastructure resources with pagination and optional filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by name/description
  - type ('nomad'|'kubernetes'|'server'|'vm'|'container'): Filter by infrastructure type
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of infrastructure resources.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      type: z.enum(["nomad", "kubernetes", "server", "vm", "container"]).optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;
      if (params.type) queryParams.type = params.type;

      const data = await apiClient.get<PaginatedResponse<Infrastructure>>("/infra", queryParams);

      const formatted = formatPaginatedList(data, formatInfrastructureListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

// ============================================================================
// DOMAIN, PEOPLE, HEALTHCHECK TOOLS
// ============================================================================

server.registerTool(
  "auto_list_domains",
  {
    title: "List Domains",
    description: `List all domains with pagination and optional search filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by FQDN
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of domains.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;

      const data = await apiClient.get<PaginatedResponse<Domain>>("/domains", queryParams);

      const formatted = formatPaginatedList(data, formatDomainListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_list_people",
  {
    title: "List People",
    description: `List all people in the system with pagination and optional filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by name/email
  - is_active (boolean): Filter by active status
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of people.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      is_active: z.boolean().optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;
      if (params.is_active !== undefined) queryParams.is_active = params.is_active;

      const data = await apiClient.get<PaginatedResponse<Person>>("/people", queryParams);

      const formatted = formatPaginatedList(data, formatPersonListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_list_healthchecks",
  {
    title: "List Healthchecks",
    description: `List all healthchecks with pagination and optional filtering.

Args:
  - page (number): Page number, default 1
  - per_page (number): Items per page (1-100), default 50
  - search (string): Filter by name
  - application_id (string): Filter by application UUID
  - service_id (string): Filter by service UUID
  - is_enabled (boolean): Filter by enabled status
  - response_format ('markdown'|'json'): Output format, default 'markdown'

Returns:
  Paginated list of healthchecks.`,
    inputSchema: z.object({
      page: z.number().int().min(1).default(1),
      per_page: z.number().int().min(1).max(100).default(50),
      search: z.string().optional(),
      application_id: z.string().uuid().optional(),
      service_id: z.string().uuid().optional(),
      is_enabled: z.boolean().optional(),
      response_format: z.nativeEnum(ResponseFormat).default(ResponseFormat.MARKDOWN)
    }).strict(),
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const queryParams: Record<string, any> = {
        page: params.page,
        per_page: params.per_page
      };
      if (params.search) queryParams.search = params.search;
      if (params.application_id) queryParams.application_id = params.application_id;
      if (params.service_id) queryParams.service_id = params.service_id;
      if (params.is_enabled !== undefined) queryParams.is_enabled = params.is_enabled;

      const data = await apiClient.get<PaginatedResponse<Healthcheck>>("/healthchecks", queryParams);

      const formatted = formatPaginatedList(data, formatHealthcheckListItem, params.response_format);
      const { content } = truncateIfNeeded(formatted, data);

      return {
        content: [{ type: "text", text: content }],
        ...(params.response_format === ResponseFormat.JSON && { structuredContent: data })
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

server.registerTool(
  "auto_execute_healthcheck",
  {
    title: "Execute Healthcheck",
    description: `Manually execute a healthcheck and return the results.

Args:
  - id (string): Healthcheck ID (UUID format)

Returns:
  Healthcheck execution result including success status, response time, status code, and any errors.`,
    inputSchema: z.object({
      id: z.string().uuid()
    }).strict(),
    annotations: {
      readOnlyHint: false,
      destructiveHint: false,
      idempotentHint: false,
      openWorldHint: true
    }
  },
  async (params) => {
    try {
      const result = await apiClient.get<Record<string, unknown>>(`/healthchecks/${params.id}/execute`);
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        structuredContent: result
      };
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }]
      };
    }
  }
);

// ============================================================================
// MAIN SERVER STARTUP
// ============================================================================

async function runStdio() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Admin Auto MCP server running via stdio");
  console.error(`Connected to: ${AUTO_URL}`);
}

async function runHTTP() {
  const app = express();
  app.use(express.json());

  app.post('/mcp', async (req: express.Request, res: express.Response) => {
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true
    });
    res.on('close', () => transport.close());
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  });

  const port = parseInt(process.env.PORT || '3000');
  app.listen(port, () => {
    console.error(`Admin Auto MCP server running on http://localhost:${port}/mcp`);
    console.error(`Connected to: ${AUTO_URL}`);
  });
}

// Choose transport based on environment
const transport = process.env.TRANSPORT || 'stdio';
if (transport === 'http') {
  runHTTP().catch(error => {
    console.error("Server error:", error);
    process.exit(1);
  });
} else {
  runStdio().catch(error => {
    console.error("Server error:", error);
    process.exit(1);
  });
}
