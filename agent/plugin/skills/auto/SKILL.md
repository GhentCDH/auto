---
name: auto
description: Look up and update the organization's IT inventory (auto) with the `auto` CLI. Use when a question involves the organization's servers, VMs, applications, services, domains or DNS, network shares, tech stacks, healthchecks or Uptime Kuma monitors, or who owns or maintains something ("where does X run", "who maintains Y", "which domains point to Z", "what runs on this VM"). Also use after the user changes infrastructure in the session (deploys or moves an app, adds or removes a VM, changes DNS, adds a share, decommissions a service), to offer to update auto so it stays correct.
---

# auto

auto is the organization's source of truth for IT infrastructure: what runs
where, who maintains it, and how it is monitored. The `auto` CLI talks to the
auto server with the user's own account, so it can only do what their role
allows (viewers can only read).

## Before the first call

Run `auto whoami --json`.

- If the command is missing, tell the user to install the `auto` binary (see
  docs/agents.md in the auto repository).
- If it reports "not signed in" or "no server url configured", ask the user to
  run `! auto login --url <their auto server>`. That opens a browser, so the
  user must run it, not you.

## Reading

Always pass `--json` and parse the result. Errors go to stderr, exit code 1.

| Need | Command |
|---|---|
| Find anything by text | `auto search <text> --json` |
| List one kind | `auto list <entity> [-s <text>] [-F key=value] [--all] --json` |
| One entity with all relations | `auto get <entity> <id-or-name> --json` |
| Short summary of an app or service | `auto overview applications <id-or-name>` |
| Live DNS records of a domain | `auto dns <domain> --json` |

Entities: run `auto schema` for the list (applications, services, infra,
domains, people, shares, stacks, healthchecks, notes). Ids can be full ids, id
prefixes, or exact names. If a name matches more than one entity (for example
the same app in prd and tst), the error lists the ids to pick from.

## Writing: always confirm first

Never change auto without the user's explicit yes for that change.

1. Run `auto schema <entity> --json`. It lists create and update fields, server
   defaults, allowed values, list filters, and the link relations with their
   fields. Do not guess field names.
2. Check the current state with `auto get` so you do not create a duplicate.
3. Show the user the exact change. Run it with `--dry-run` first to show the
   request that will be sent.
4. After the user agrees, run the same command without `--dry-run`.

| Change | Command |
|---|---|
| Create | `auto create <entity> key=value ... --json` |
| Update (only given fields) | `auto update <entity> <id-or-name> key=value ... --json` |
| Link | `auto link <entity> <id> <relation> <other-id> [key=value ...] --json` |
| Unlink | `auto unlink <entity> <id> <relation> <other-id> --json` |
| Delete | `auto delete <entity> <id-or-name> --yes --json` |

Values are typed from the schema (`is_active=false`, `port=443`,
`manual_ips=10.0.0.1,10.0.0.2`). Use `key:=<json>` for a raw JSON value, or
`-d '<json object>'` for a whole body. Links are made from one side only (for
example `applications ... people ...`); the error tells you when to swap.

## Keeping auto in sync

When the user has changed infrastructure in this session, compare what they
did with what auto has (`auto get` / `auto search`). Then list the
differences and propose the commands to fix them, for example a new VM to
create and link to the application, a changed URL, or a service to mark
`status=deprecated`. Wait for confirmation before you run them.

## More

`auto --help` and `auto <command> --help` show every command, including
`auto sync ...`, `auto check <healthcheck>` and `auto api <METHOD> <path>` for
endpoints without their own command.
