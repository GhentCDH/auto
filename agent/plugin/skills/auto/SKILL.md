---
name: auto
description: Look up and update the organization's IT inventory (auto) with the `auto` CLI, and read the deployment config behind it (Caddy proxy and load balancer config, Nomad job files). Use when a question involves the organization's websites, applications, services, servers, VMs, domains or DNS, proxy or load balancer setup, how a site is served or deployed, network shares, tech stacks, healthchecks or uptime ("what is down", "where does X run", "how is X proxied", "who maintains Y", "which domains point to Z", "what runs on this VM"). Also use after the user changes infrastructure in the session (deploys or moves an app, adds or removes a VM, changes DNS, adds a share, decommissions a service), to offer to update auto so it stays correct.
---

# auto

auto is the organization's inventory of IT infrastructure: what runs where,
who maintains it, and how it is monitored. The `auto` CLI talks to the auto
server as the user. Everything you need is on this page: do not run
`auto --help`, and do not read the OpenAPI document.

Run commands directly. Output is JSON when piped (add `--json` to be sure).
Errors go to stderr with exit code 1, and they explain what to do:

- `command not found`, or `unrecognized subcommand` (the installed `auto` is
  older than this page): the user must install or update `auto`:
  `curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh`
- `not signed in` or `no server url configured`: ask the user to run
  `! auto login --url <server>` (it opens a browser, so the user runs it).
- `not allowed`: the user's role cannot do this (viewers only read).

## Common questions

| Question | Command |
|---|---|
| What is down / are there problems? | `auto status --problems` |
| Status of everything (uptime) | `auto status` |
| Find something by name or text | `auto search <text>` |
| Where does app X run? Who maintains it? Its domains, healthchecks? | `auto get applications <name>` |
| What runs on server/VM X? Which domains point to it? | `auto get infra <name>` |
| Which apps does person X work on? | `auto get people <name>` |
| Short summary of an app or service | `auto overview applications <name>` |
| All apps in production | `auto list applications -F environment=prd --all` |
| Live DNS records of a domain | `auto dns <fqdn>` |
| Run one healthcheck now | `auto check <healthcheck>` |
| Which app or service is this git repo? | `auto here` |
| Is auto complete and up to date for this repo/app? | use the `auto:audit` skill |
| How is X served? Proxy, load balancer, Nomad job? | see "How a site is served" below |

## How a site is served (proxy, load balancer, Nomad)

auto says which systems are involved; the deploy repos say how. Do not stop
at auto: read the config, and answer with the concrete chain (domain, DNS,
load balancer, upstream host and port, container or Nomad job), citing files.

1. In auto: `auto get domains <fqdn>` (its target and `ips`) or
   `auto get applications <name>` (its `domains` and `infra`). The load
   balancer is a service in auto; its `repository_url` is its config repo.
2. Local checkouts: `auto deploy-repos --fetch`. It lists the configured
   checkouts with what they hold (`caddy`, `nomad`) and whether they are
   `behind` their remote.
   - Read the remote version, not a stale working tree:
     `git -C <path> grep -n -e <fqdn> -e <name> origin/<branch>` and
     `git -C <path> show origin/<branch>:<file>`. Mention local changes
     (`dirty`) if they matter. Pull only when the user asks.
   - No deploy repos configured: ask the user for the paths of their
     checkouts of the Caddy config and Nomad jobs repos, and offer to add them
     as `deploy_repos = [...]` in `~/.config/auto/config.toml`.
3. Read the repo's `CLAUDE.md` or `README.md` first for its layout. Current
   layouts:
   - **Caddy** (`caddy`): one file per site, named by the reversed domain
     (`auto.ghentcdh.be` is `be.ghentcdh.auto.caddy`), under
     `hosts/<host>/sites-enabled/` (live) and `sites-available/` (defined).
     A domain can have one file per hop: production sites usually pass the
     load balancer (`hosts/lbprd`), which forwards to a docker host
     (`hosts/dockprd1`) whose Caddy routes to the container; some hosts, such
     as dockqas1, are reached directly. The grep above finds every hop.
   - **Nomad** (`nomad`): one folder per app with `job.nomad.hcl.j2` (a
     template) and one `<env>.vars.yaml` per environment, which holds most
     values (domains, image, ports). Nomad apps do not pass the Caddy load
     balancer.

`<name>` can be an exact name, a full id or an id prefix. If a name matches
several entities (for example one app in prd and in tst), the error lists the
ids: pick one and run the command again.

## Entities and output shapes

Entities: `applications`, `services`, `infra`, `domains`, `people`, `shares`,
`stacks`, `healthchecks`, `notes`. Singular forms and `app`, `vm`, `hc` also
work.

- `auto list <entity> [-s <text>] [-F key=value ...] [--all]` returns
  `{data: [...], total, page, per_page, total_pages}`. Without `--all` you get
  one page of 50. Rows embed their relations, like `get`.
  Filters: `status` and `environment` for applications and services.
- `auto get <entity> <name>` returns the entity's own fields plus relation
  arrays:
  - applications: `infra`, `services`, `domains`, `people` (with
    `contribution_type`: developer, maintainer, project_owner, stakeholder),
    `network_shares`, `stacks`, `healthchecks`, `notes`
  - services: `applications`, `infra`, `healthchecks`
  - infra: `ips`, `domain`, `applications`, `services`, `healthchecks`,
    `pointing_domains`
  - domains: `target_application_name` / `target_service_name` /
    `target_infra_name`, `applications`, `ips`
  - people, shares, stacks: `applications`
- `auto search <text>` returns
  `{applications: [...], services: [...], infra: [...], domains: [...], people: [...], network_shares: [...], stacks: [...], healthchecks: [...]}`,
  each hit `{id, name, description, entity_type}`.
- `auto status [--problems | -s down,pending]` returns an array, worst first,
  of `{id, name, state, url, application_name, service_name, since, last_check, message, ping_ms, uptime_percent}`.
  `state` is one of down, pending, unknown (no data from Uptime Kuma),
  maintenance, up, disabled. `--problems` means down, pending and unknown.
  `since` is limited to the last hour, which is all the server keeps.
- Common values: environment `dev`, `tst`, `qas`, `prd`; status `active`,
  `inactive`, `deprecated`, `archived`.

## Changing data: always confirm first

Never change auto without the user's explicit yes for that exact change.

1. Look at the current state (`auto get` / `auto search`), so you do not
   create a duplicate.
2. Get the field names: `auto schema <entity>`. It returns
   `{entity, create: [field], update: [field], list_filters: [field], relations: [{name, fields: [field]}]}`
   with `field = {name, type, required, default, enum}`. Do not guess names.
3. Show the user the change, with the command and `--dry-run` output.
4. After the user agrees, run it without `--dry-run`.

| Change | Command |
|---|---|
| Create | `auto create <entity> key=value ...` |
| Update (only the given fields) | `auto update <entity> <name> key=value ...` |
| Link | `auto link <entity> <name> <relation> <other> [key=value ...]` |
| Unlink | `auto unlink <entity> <name> <relation> <other>` |
| Delete | `auto delete <entity> <name> --yes` |

- Values are typed from the schema: `is_enabled=false`, `interval=60`,
  `manual_ips=10.0.0.1,10.0.0.2`, `key=null` for nullable fields.
  `key:=<json>` passes raw JSON; `-d '<json object>'` passes a whole body.
- Links go from applications to infra, services, domains, people, shares and
  stacks, and from services to infra. The other direction gives an error that
  says how to swap.
- A domain needs exactly one target: `target_application_id`,
  `target_service_id` or `target_infra_id`. A healthcheck needs `domain_id`
  and one of `application_id` or `service_id`.

## Keeping auto in sync

When the user has changed infrastructure in this session, compare it with
auto (`auto get` / `auto search`). List the differences and propose the
commands, for example a new VM to create and link to the application, a
changed URL, or `status=deprecated` for a service that was switched off.
Wait for confirmation before you run them. For a full check of the
application or service in the current repo, use the `auto:audit` skill.

## Other commands

`auto sync infra|kuma|domains [id]` and
`auto sync outline applications|services <name>` start server-side syncs.
`auto whoami` shows the user and role. `auto token create <name> --save`
creates an API token. `auto api <METHOD> <path> [-d json]` calls any API
endpoint that has no command of its own.
