# Usage Guide: Admin Auto MCP Server

## Quick Start

### 1. Set Up Configuration

Create a `.env` file with your Admin Auto credentials:

```bash
cp .env.example .env
```

Edit `.env` with your actual credentials:

```bash
AUTO_URL=https://auto.ghentcdh.be
AUTO_USERNAME=your-username
AUTO_PASSWORD=your-password
```

### 2. Test the Connection

Test connectivity using the dashboard stats tool:

```bash
# Set environment variables
export AUTO_URL=https://auto.ghentcdh.be
export AUTO_USERNAME=your-username
export AUTO_PASSWORD=your-password

# Run the server (it expects MCP protocol messages on stdin)
node dist/index.js
```

## Integration with Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS, `%APPDATA%\Claude\claude_desktop_config.json` on Windows):

```json
{
  "mcpServers": {
    "admin-auto": {
      "command": "node",
      "args": [
        "/home/miepeete/code/admin/auto/admin-auto-mcp-server/dist/index.js"
      ],
      "env": {
        "AUTO_URL": "https://auto.ghentcdh.be",
        "AUTO_USERNAME": "your-username",
        "AUTO_PASSWORD": "your-password"
      }
    }
  }
}
```

After adding the configuration, restart Claude Desktop.

## Available Tools

### 🔍 Search & Discovery

#### `auto_global_search`
Search across all resource types in one query.

**Example prompts:**
- "Search for everything related to GitLab"
- "Find docker resources"
- "Show me kubernetes infrastructure"

**Parameters:**
- `query` (string, required): Search term
- `response_format` ('markdown'|'json'): Output format

#### `auto_get_dashboard_stats`
Get aggregated counts and statistics.

**Example prompt:**
- "Show me the dashboard statistics"

### 📱 Applications

#### `auto_list_applications`
List applications with optional filtering.

**Example prompts:**
- "List all production applications"
- "Show me applications in development environment"
- "Find applications with 'api' in the name"

**Parameters:**
- `page` (number): Page number
- `per_page` (number): Items per page (1-100)
- `search` (string): Filter by name/description
- `environment` ('prd'|'dev'|'stg'|'tst'): Filter by environment
- `status` ('active'|'inactive'|'deprecated'): Filter by status
- `response_format` ('markdown'|'json'): Output format

#### `auto_get_application`
Get detailed information about a specific application.

**Example prompt:**
- "Show me details for application {uuid}"

**Parameters:**
- `id` (string, required): Application UUID
- `response_format` ('markdown'|'json'): Output format

**Returns:** Complete application details including:
- Infrastructure associations
- Connected services
- Domains
- People/contributors
- Network shares
- Technology stacks
- Healthchecks
- Notes

#### `auto_create_application`
Create a new application.

**Example prompt:**
- "Create a new application called 'My API' in production"

**Parameters:**
- `name` (string, required): Application name
- `description` (string): Description
- `repository_url` (string): Git repository URL
- `environment` ('prd'|'dev'|'stg'|'tst'): Environment (default: 'prd')
- `url` (string): Application URL
- `status` ('active'|'inactive'|'deprecated'): Status (default: 'active')

### 🔧 Services

#### `auto_list_services`
List services with optional filtering.

**Example prompts:**
- "List all services in production"
- "Show me database services"

**Parameters:** Same as `auto_list_applications`

### 🖥️ Infrastructure

#### `auto_list_infrastructure`
List infrastructure resources (Nomad clusters, Kubernetes, servers, VMs).

**Example prompts:**
- "Show me all Kubernetes clusters"
- "List Nomad infrastructure"
- "Find all servers"

**Parameters:**
- `page` (number): Page number
- `per_page` (number): Items per page (1-100)
- `search` (string): Filter by name/description
- `type` ('nomad'|'kubernetes'|'server'|'vm'|'container'): Filter by type
- `response_format` ('markdown'|'json'): Output format

### 🌐 Domains

#### `auto_list_domains`
List registered domains.

**Example prompts:**
- "List all domains"
- "Find domains containing 'api'"

**Parameters:**
- `page` (number): Page number
- `per_page` (number): Items per page (1-100)
- `search` (string): Filter by FQDN
- `response_format` ('markdown'|'json'): Output format

### 👥 People

#### `auto_list_people`
List people and team members.

**Example prompts:**
- "List all active developers"
- "Show me all people"

**Parameters:**
- `page` (number): Page number
- `per_page` (number): Items per page (1-100)
- `search` (string): Filter by name/email
- `is_active` (boolean): Filter by active status
- `response_format` ('markdown'|'json'): Output format

### ❤️ Healthchecks

#### `auto_list_healthchecks`
List configured healthchecks.

**Example prompts:**
- "List all healthchecks"
- "Show me healthchecks for application {uuid}"
- "Find enabled healthchecks"

**Parameters:**
- `page` (number): Page number
- `per_page` (number): Items per page (1-100)
- `search` (string): Filter by name
- `application_id` (string): Filter by application UUID
- `service_id` (string): Filter by service UUID
- `is_enabled` (boolean): Filter by enabled status
- `response_format` ('markdown'|'json'): Output format

#### `auto_execute_healthcheck`
Manually execute a healthcheck and get results.

**Example prompt:**
- "Execute healthcheck {uuid}"

**Parameters:**
- `id` (string, required): Healthcheck UUID

**Returns:**
- Success status
- HTTP status code
- Response time in milliseconds
- Body match result
- Any errors

## Example Workflows

### 1. Finding and Exploring an Application

```
User: "Search for GitLab"
→ Tool: auto_global_search(query="gitlab")

User: "Show me details for the GitLab application"
→ Tool: auto_get_application(id="<uuid-from-search>")
```

### 2. Infrastructure Audit

```
User: "List all Kubernetes clusters"
→ Tool: auto_list_infrastructure(type="kubernetes")

User: "Show me all production applications"
→ Tool: auto_list_applications(environment="prd")
```

### 3. Healthcheck Monitoring

```
User: "List all healthchecks"
→ Tool: auto_list_healthchecks()

User: "Execute healthcheck for API endpoint"
→ Tool: auto_execute_healthcheck(id="<uuid>")
```

### 4. Creating Resources

```
User: "Create a new application called 'Customer Portal'"
→ Tool: auto_create_application(
    name="Customer Portal",
    description="Customer-facing portal application",
    environment="prd",
    status="active"
  )
```

## Tips

1. **Use Global Search First**: When you're unsure what you're looking for, start with `auto_global_search` - it searches across all resource types.

2. **Pagination**: Large result sets are paginated. Use `page` and `per_page` parameters to navigate through results.

3. **Filtering**: Most list tools support filtering. Use `search`, `environment`, `status`, and `type` parameters to narrow down results.

4. **Response Formats**:
   - Use **markdown** (default) for human-readable output
   - Use **json** for programmatic processing or when you need all fields

5. **Getting IDs**: Use list tools to find resource IDs, then use detail tools (like `auto_get_application`) to get complete information.

## Troubleshooting

### "ERROR: Missing required environment variables"

Make sure you've set all three required environment variables:
- `AUTO_URL`
- `AUTO_USERNAME`
- `AUTO_PASSWORD`

### "Authentication failed"

Check that your username and password are correct for the Admin Auto instance.

### "Cannot connect to Admin Auto instance"

Verify:
- The `AUTO_URL` is correct
- The Admin Auto instance is accessible from your network
- You're using HTTPS (not HTTP) for production instances

### "Resource not found"

The UUID you provided doesn't exist. Use a list tool to find the correct UUID first.

## Security Notes

- **Never commit** your `.env` file or credentials to version control
- Use HTTPS URLs for production Admin Auto instances
- Store credentials securely (environment variables or secret management)
- The MCP server runs locally and connects to Admin Auto on your behalf
- HTTP Basic Auth credentials are sent with every request (over HTTPS)
