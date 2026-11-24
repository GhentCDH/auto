---
name: setup
description: Install and configure the auto CLI and its MCP server for the user, step by step, doing as much as possible for them. Use when the user asks to set up, install, configure, sign in to or fix auto, its MCP server or this plugin; when the auto_* MCP tools are missing or the auto MCP server failed to start; when a tool says "not signed in" or "no server url configured"; or when the user wants to add their deploy repos (Caddy config, Nomad jobs) to auto.
---

# Set up auto

The auto plugin needs three things on the user's machine: the `auto`
binary, a server URL with a saved API token, and (for admins) the paths of
their deploy repo checkouts. Find out what is missing, explain it in a few
sentences, and do the steps for the user. Ask before you install, clone or
change a file. Skip steps that are already done.

The organization's server is `https://auto.ghentcdh.be`.

## 1. Check the current state

Run this yourself, in one Bash call. It counts keys in the config file and
never prints it, because the file holds the token:

```sh
command -v auto && auto --version
auto whoami --json
f=~/.config/auto/config.toml; [ -f "$f" ] || f="$HOME/Library/Application Support/auto/config.toml"
echo "token: $(grep -c '^token' "$f" 2>/dev/null)  deploy_repos: $(grep -c '^deploy_repos' "$f" 2>/dev/null)"
```

| Result | Meaning | Go to |
|---|---|---|
| `auto` not found | not installed, or not on `PATH` | step 2 |
| `no server url configured` | installed, never signed in | step 3 |
| `not signed in` or `the configured API token was rejected` | no valid login or token | step 3 |
| JSON with `role`, but `token: 0` | signed in with a browser session, which expires after a week | step 3, "Save a token" |
| JSON with `role` and `token: 1` | ready | step 4 |

Tell the user in one or two sentences what is missing and what you will do.

## 2. Install the binary

Ask first, then run:

```sh
curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh
```

It installs `auto` to `~/.local/bin` (Linux x86_64/arm64, macOS). When it
prints `is not on your PATH`, offer to add
`export PATH="$HOME/.local/bin:$PATH"` to the user's shell profile
(`~/.profile`, `~/.zshrc` or `~/.bashrc`, the one their shell reads). Claude
Code only sees the new `PATH` after a restart from a new shell.

## 3. Sign in and save a token

**Sign in (the user does this):** the login opens a browser for the password
or SSO login, which you cannot complete. Ask the user to type:

```
! auto login --url https://auto.ghentcdh.be
```

The `!` runs it in this session. When it prints `signed in`, continue.

**Save a token (you do this):** the MCP server and the hooks use a personal
API token, because the browser session expires. Ask first, then run, with
stdout discarded so the secret does not enter the conversation:

```sh
auto token create "claude $(hostname)" --save >/dev/null
auto whoami --json
```

The note on stderr names the config file it wrote (`~/.config/auto/config.toml`
on Linux, `~/Library/Application Support/auto/config.toml` on macOS). Never
print or read that file whole: it holds the token.

A token has its owner's role. `whoami` shows `role`: `admin`, `editor` or
`viewer`. A viewer can read everything and change nothing.

## 4. Connect the MCP server

After an install or a first sign-in, the auto MCP server must start again:
ask the user to run `/mcp`, select `auto` and choose reconnect, or to restart
Claude Code (needed after a `PATH` change). Then check it with one call:
`auto_status` with `problems: true`.

If the tools show up twice, the user also added the server by hand before.
Offer to remove that copy: `claude mcp remove auto -s user`.

## 5. Deploy repos (admins only)

Skip this step unless `role` is `admin`.

auto says which systems serve an app; two central repos say how. With local
checkouts, Claude can answer "how is X proxied or deployed" from the real
config:

| Repo | Remote |
|---|---|
| Caddy proxy and load balancer config | `git@github.com:GhentCDH/caddy-config.git` |
| Nomad jobs | `git@github.ugent.be:containers/nomad-jobs-ns-ghentcdh.git` |

1. Step 1 showed `deploy_repos: 1` when the setting exists: then show it
   with `auto deploy-repos` and stop.
2. Ask if the user has checkouts of these repos, and where. If not, offer to
   clone them into a folder they choose (cloning needs their SSH access to
   both hosts). If they do not want them, set `deploy_repos = []`: they are
   then not asked again.
3. Ask first, then append the setting (with `$f` from step 1). The file has
   only top-level keys, so appending is safe:
   ```sh
   printf '\ndeploy_repos = [\n  "%s",\n  "%s",\n]\n' "~/path/to/caddy-config" "~/path/to/nomad-jobs-ns-ghentcdh" >>"$f"
   ```
4. Check it: `auto deploy-repos --fetch` lists both repos with their branch.

## 6. Optional: fewer permission prompts

Every read tool asks for permission the first time. Offer to allow the
read-only tools in the user's `~/.claude/settings.json` under
`permissions.allow` (skip this when the organization's managed settings
already do it):

```
mcp__plugin_auto_auto__auto_list, mcp__plugin_auto_auto__auto_get,
mcp__plugin_auto_auto__auto_search, mcp__plugin_auto_auto__auto_status,
mcp__plugin_auto_auto__auto_schema, mcp__plugin_auto_auto__auto_overview,
mcp__plugin_auto_auto__auto_here, mcp__plugin_auto_auto__auto_deploy_repos
```

Tools that change data or reach live systems keep asking.

## Done

End with a short summary: what was installed or configured, the user's role,
and one example question to try ("what is down?", "where does <app> run?").
Updates later: run the install command again; `auto` says when the server is
newer.
