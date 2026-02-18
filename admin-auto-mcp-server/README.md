# Admin Auto MCP Server

MCP (Model Context Protocol) server for interacting with Admin Auto instances. Provides comprehensive access to infrastructure management data including applications, services, infrastructure resources, domains, people, healthchecks, and more.

## Features

- **Global Search**: Search across all resource types
- **Applications Management**: List, view, and create applications
- **Services Management**: List and manage services
- **Infrastructure Management**: Track servers, Kubernetes clusters, Nomad clusters, VMs
- **Domain Management**: Track domain registrations and DNS
- **People Management**: Track team members and contributors
- **Healthcheck Management**: List and execute healthchecks
- **Dashboard Statistics**: Get aggregated stats

## Installation

```bash
# Install dependencies
npm install

# Build the TypeScript project
npm run build
```

## Configuration

The server requires HTTP Basic Authentication credentials for the Admin Auto instance.

### Option 1: Using .env file (Recommended for local testing)

Create a `.env` file in the project root:

```bash
# Copy the example file
cp .env.example .env

# Edit with your credentials
nano .env
```

Contents of `.env`:
```bash
AUTO_URL=https://auto.ghentcdh.be
AUTO_USERNAME=your-username
AUTO_PASSWORD=your-password
TRANSPORT=stdio
```

The `dotenv` package automatically loads these when you run `npm start`.

### Option 2: Environment variables (Recommended for Claude Desktop)

Set environment variables directly in your shell:

```bash
export AUTO_URL=https://auto.ghentcdh.be
export AUTO_USERNAME=your-username
export AUTO_PASSWORD=your-password
```

Or specify them in Claude Desktop's MCP configuration (see below).

### Option 3: Claude Desktop MCP Configuration (Best for production use)

Credentials are specified in the MCP config file (most secure for Claude Desktop):

```json
{
  "mcpServers": {
    "admin-auto": {
      "command": "node",
      "args": ["/path/to/dist/index.js"],
      "env": {
        "AUTO_URL": "https://auto.ghentcdh.be",
        "AUTO_USERNAME": "your-username",
        "AUTO_PASSWORD": "your-password"
      }
    }
  }
}
```

**Important:** When running via Claude Desktop, the `.env` file is NOT used. Credentials come from the `env` section of the MCP configuration.

## Usage

### Stdio Transport (Local/CLI)

```bash
npm start
```

Or run directly:

```bash
node dist/index.js
```

### HTTP Transport (Remote Server)

```bash
TRANSPORT=http PORT=3000 npm start
```

The server will be available at `http://localhost:3000/mcp`

## Available Tools

### Search & Dashboard

- `auto_global_search` - Search across all resource types
- `auto_get_dashboard_stats` - Get dashboard statistics and counts

### Applications

- `auto_list_applications` - List applications with filtering and pagination
- `auto_get_application` - Get detailed application information with all relations
- `auto_create_application` - Create a new application

### Services

- `auto_list_services` - List services with filtering and pagination

### Infrastructure

- `auto_list_infrastructure` - List infrastructure resources (Nomad, Kubernetes, servers, VMs)

### Domains

- `auto_list_domains` - List registered domains

### People

- `auto_list_people` - List people and team members

### Healthchecks

- `auto_list_healthchecks` - List configured healthchecks
- `auto_execute_healthcheck` - Execute a healthcheck manually

## Output Formats

Most tools support two output formats:

- **Markdown** (default): Human-readable formatted text
- **JSON**: Machine-readable structured data for programmatic processing

Specify format using the `response_format` parameter:

```json
{
  "response_format": "json"
}
```

## Pagination

List tools support pagination with these parameters:

- `page` - Page number (default: 1)
- `per_page` - Items per page, max 100 (default: 50)

## Filtering

List tools support various filters:

- `search` - Text search across names and descriptions
- `environment` - Filter by environment (prd, dev, stg, tst)
- `status` - Filter by status (active, inactive, deprecated)
- `type` - Filter by infrastructure type
- `is_active` - Filter by active status (people)
- `is_enabled` - Filter by enabled status (healthchecks)

## Development

```bash
# Install dependencies
npm install

# Run in development mode with auto-reload
npm run dev

# Build for production
npm run build

# Clean build artifacts
npm run clean
```

## Claude Desktop Integration

Add to your Claude Desktop MCP settings (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "admin-auto": {
      "command": "node",
      "args": ["/path/to/admin-auto-mcp-server/dist/index.js"],
      "env": {
        "AUTO_URL": "https://auto.ghentcdh.be",
        "AUTO_USERNAME": "your-username",
        "AUTO_PASSWORD": "your-password"
      }
    }
  }
}
```

## Security Considerations

- **Credentials**: Store credentials securely in environment variables or `.env` files
- **HTTP Basic Auth**: The server uses HTTP Basic Authentication to connect to Admin Auto
- **Permissions**: Ensure the MCP server has appropriate read/write permissions for your use case
- **HTTPS**: Always use HTTPS URLs for production Admin Auto instances

## Admin Auto API

This MCP server connects to an Admin Auto instance API. For complete API documentation, see the Admin Auto API reference.

Key endpoints:
- `/api/applications` - Application management
- `/api/services` - Service management
- `/api/infra` - Infrastructure resources
- `/api/domains` - Domain registrations
- `/api/people` - People and team members
- `/api/healthchecks` - Healthcheck monitoring
- `/api/search` - Global search
- `/api/dashboard/stats` - Dashboard statistics

## License

MIT

## Author

Created for the Admin Auto infrastructure management system.
