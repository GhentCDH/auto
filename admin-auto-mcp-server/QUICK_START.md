# Quick Start Guide

## 1. Configure Credentials

**For Local Testing:**
```bash
cd admin-auto-mcp-server

# Copy example environment file
cp .env.example .env

# Edit with your credentials
nano .env
```

Required variables in `.env`:
```bash
AUTO_URL=https://auto.ghentcdh.be
AUTO_USERNAME=your-username
AUTO_PASSWORD=your-password
```

**For Claude Desktop:** Credentials go in the MCP config (step 3 below), NOT in `.env`

## 2. Build the Server

```bash
npm install
npm run build
```

## 3. Integrate with Claude Desktop

### macOS
Edit: `~/Library/Application Support/Claude/claude_desktop_config.json`

### Linux
Edit: `~/.config/Claude/claude_desktop_config.json`

### Windows
Edit: `%APPDATA%\Claude\claude_desktop_config.json`

### Configuration

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

## 4. Restart Claude Desktop

Close and reopen Claude Desktop to load the MCP server.

## 5. Test It Out

Try these prompts in Claude:

- "Search for GitLab in Auto"
- "List all production applications"
- "Show me dashboard statistics"
- "Find Kubernetes infrastructure"
- "List all healthchecks"

## Available Tools (11 total)

### 🔍 Search
- `auto_global_search` - Search everything
- `auto_get_dashboard_stats` - Get stats

### 📱 Applications
- `auto_list_applications` - List apps
- `auto_get_application` - Get app details
- `auto_create_application` - Create app

### 🔧 Services
- `auto_list_services` - List services

### 🖥️ Infrastructure
- `auto_list_infrastructure` - List infra

### 🌐 Other
- `auto_list_domains` - List domains
- `auto_list_people` - List people
- `auto_list_healthchecks` - List healthchecks
- `auto_execute_healthcheck` - Run healthcheck

## Troubleshooting

### "Missing required environment variables"
→ Check your `.env` file or Claude config `env` section

### "Authentication failed"
→ Verify username and password are correct

### "Cannot connect"
→ Check `AUTO_URL` and network connectivity

### Tools not showing in Claude
→ Restart Claude Desktop after config changes

## Next Steps

- See **USAGE.md** for detailed examples
- See **README.md** for full documentation
- See **PROJECT_SUMMARY.md** for technical details
