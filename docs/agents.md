# auto for the terminal and for agents

The `auto` binary is the client for an auto server:

- `auto` opens the read-only TUI.
- `auto <command>` reads and writes through the API (tables on a terminal,
  JSON when piped or with `--json`).
- `auto mcp` runs an MCP server on stdio for agents.

Every request runs as your own account, so you can only do what your role
allows. A viewer can read everything but change nothing.

## 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh
```

The script picks the archive for your platform from the latest
[GitHub release](https://github.com/GhentCDH/auto/releases), checks its
SHA-256, and installs `auto` to `~/.local/bin` (set `AUTO_INSTALL_DIR` to
change that). Pass a version to pin one: `sh install.sh v1.15.0`.

Platforms: Linux x86_64 and arm64 (static musl builds), macOS Apple Silicon
and Intel. When the release has no archive for your platform, the script
builds the CLI from the release tag with `cargo` (install it with
[rustup](https://rustup.rs)).

### Updates

The CLI has the same version as the server, because one release builds both.
Once a day, `auto` compares its version with the server's. When the server is
newer, it prints a note with the install command. Run that command again to
update.

Most server releases need no CLI update: fields, allowed values and link
relations are read from the server at runtime. Update when you want new
commands or fixes.

## 2. Sign in

```bash
auto login --url auto.example.org   # saves the URL, opens the browser
auto whoami
```

`auto login` works for password and SSO accounts. The session lasts the
server's `SESSION_HOURS` (default 7 days), counted from the login: using it
does not extend it.

Agents and MCP clients should use a personal API token, which does not expire
unless you ask it to:

```bash
auto token create "claude laptop" --save    # stored in ~/.config/auto/config.toml
```

You can list and revoke tokens with `auto token list` / `auto token revoke <id>`,
or in the web UI under Settings. A token has the role of its owner. For an
agent that should only read, create the token on a viewer account.

Configuration is read from `~/.config/auto/config.toml` (`url`, `token`), and
the environment variables `AUTO_URL` and `AUTO_TOKEN` override it.

If you used the older `auto-tui`, move its config once; it is not read
anymore:

```bash
mv ~/.config/auto-tui ~/.config/auto
```

## 3. Use it

```bash
auto search wiki
auto list apps -F environment=prd -F status=active
auto get infra vm-web-01               # by id, id prefix or exact name
auto schema applications               # fields, defaults, allowed values, links
auto create people name=Alice email=alice@example.org --dry-run
auto update apps wiki status=deprecated
auto link applications wiki people alice contribution_type=maintainer
auto delete stacks old-stack
auto api GET /infra/loads              # any endpoint without its own command
```

`auto --help` lists all commands.

## 4. Claude Code: install the skill

The repository is a Claude Code plugin marketplace with one plugin, `auto`.
It holds a skill that tells Claude when to consult auto (questions about
servers, apps, domains, ownership, monitoring) and to offer an update after
you change infrastructure. Claude then uses the `auto` CLI, and asks before
any write.

One user:

```
/plugin marketplace add GhentCDH/auto
/plugin install auto@auto
```

### Whole organization

Put this in the organization's Claude Code
[managed settings](https://code.claude.com/docs/en/managed-settings.md):

```json
{
  "extraKnownMarketplaces": {
    "auto": {
      "source": { "source": "github", "repo": "GhentCDH/auto" },
      "autoUpdate": true
    }
  },
  "enabledPlugins": {
    "auto@auto": true
  }
}
```

Claude Code registers the marketplace and installs the plugin at the start
of each user's next session. `autoUpdate` keeps it current: updates for a git
marketplace are off unless the managed entry turns them on. The repository is
public, so users need no git credentials.

There are two ways to deliver the settings. Use only one, because by default
only one managed source applies per machine, and server-managed settings win:

- **claude.ai admin console** (Team or Enterprise plan, Owner role):
  Organization settings > Claude Code > Managed settings
  (`https://claude.ai/admin-settings/claude-code`). It reaches everyone who
  signs in to Claude Code with an organization account. No change on machines.
- **A file on each machine**, for example through Puppet or MDM:
  - Linux: `/etc/claude-code/managed-settings.json`
  - macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
  - Windows: `C:\Program Files\ClaudeCode\managed-settings.json`

The claude.ai plugin library ("Sync from GitHub") is not an option for this
repository, because it only syncs from private or internal repositories.

The plugin has no `version`, so Claude Code follows the commit SHA. A skill
change reaches users after it is pushed to `main`, without a version bump.

Every user still needs the `auto` binary and a login (steps 1 and 2). The
skill tells Claude to run `auto whoami` first, and to ask the user to install
or sign in when that fails.

## 5. Claude Desktop and other MCP clients

Add `auto mcp` as a stdio server. Use the full path to the binary, because
desktop apps often do not see your shell `PATH`. With a token saved by
`auto token create ... --save`, no secret goes into the client config.

Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "auto": {
      "command": "/home/you/.local/bin/auto",
      "args": ["mcp"]
    }
  }
}
```

Other clients (Codex, Cursor, opencode, ...) take the same command and
arguments in their own MCP config format. To point a client at another server
or account, set `AUTO_URL` and `AUTO_TOKEN` in the server's `env`.

The MCP server has 13 tools: `auto_list`, `auto_get`, `auto_search`,
`auto_schema`, `auto_overview`, `auto_create`, `auto_update`, `auto_delete`,
`auto_link`, `auto_unlink`, `auto_dns`, `auto_run_healthcheck` and `auto_sync`.
Reads are marked read-only and deletes as destructive, so clients can ask
before a write. Every write tool takes `dry_run`.

The skill comes with the Claude Code plugin (step 4). Do not also upload it
as an organization skill on claude.ai: the Desktop Code tab loads both, so
users would get it twice. The MCP server's own instructions already tell the
model when and how to use the tools.

The claude.ai web app cannot start local stdio servers. It would need a
remote MCP endpoint on the auto server, which does not exist yet.

## How it stays in sync with the server

The CLI does not hard-code fields. It reads create and update fields, list
filters and link relations from the server's `/api/openapi.json`, and allowed
values and defaults from `/api/config`, each time it runs. A new field in the
backend shows up in `auto schema`, in `key=value` parsing and in the MCP
server, without a new CLI release. Only a new entity kind needs a CLI change
(see "Adding a New Entity" in `AGENTS.md`).
