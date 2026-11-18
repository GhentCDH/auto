---
name: auto
description: Look up and update the organization's IT inventory (auto) with the auto MCP tools, and read the deployment config behind it (Caddy proxy and load balancer config, Nomad job files). Use when a question involves the organization's websites, applications, services, servers, VMs, domains or DNS, proxy or load balancer setup, how a site is served or deployed, network shares, tech stacks, healthchecks or uptime ("what is down", "where does X run", "how is X proxied", "who maintains Y", "which domains point to Z", "what runs on this VM"). Also use after the user changes infrastructure in the session (deploys or moves an app, adds or removes a VM, changes DNS, adds a share, decommissions a service), to offer to update auto so it stays correct.
---

# auto

auto is the organization's inventory of IT infrastructure: what runs where,
who maintains it, and how it is monitored. Use the `auto_*` MCP tools from
this plugin; they act as the user, so they can only do what the user's role
allows (viewers only read). Everything you need is on this page: do not
look for other documentation.

When a tool fails, its error says what to do:

- The `auto_*` tools are missing, or the server failed to start: the user
  must install the `auto` binary:
  `curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh`,
  then restart Claude Code.
- `not signed in` or `no server url configured`: ask the user to run
  `! auto login --url <server>` (it opens a browser) and then
  `! auto token create "claude" --save`.
- `not allowed`: the user can only read (viewer). Say so, and stop
  proposing changes to auto for the rest of the session.

Many users only read: they ask where something runs or who to contact, and
do not maintain infrastructure. Answer them plainly from auto, without
setup steps they do not need.
- `the server is too old`: the auto server needs an update; say so.

## Common questions

| Question | Tool |
|---|---|
| What is down / are there problems? | `auto_status` with `problems: true` |
| Status of everything (uptime) | `auto_status` |
| Find something by name or text | `auto_search` |
| Where does app X run? Who maintains it? Its domains, healthchecks? | `auto_get` (entity `applications`) |
| What runs on server/VM X? Which domains point to it? | `auto_get` (entity `infra`) |
| Which apps does person X work on? | `auto_get` (entity `people`) |
| Short summary of an app or service | `auto_overview` |
| All apps in production | `auto_list` (entity `applications`, `filters: {environment: prd}`, `all: true`) |
| Live DNS records of a domain | `auto_dns` |
| Run one healthcheck now | `auto_run_healthcheck` |
| Which app or service is this git repo? | `auto_here` with the project's path |
| Is auto complete and up to date for this repo/app? | use the `auto:audit` skill |
| How is X served? Proxy, load balancer, Nomad job? | see "How a site is served" below |

Ids can be an exact name, a full id or an id prefix. If a name matches
several entities (for example one app in prd and in tst), the error lists the
ids: pick one and call the tool again.

## Entities and results

Entities: `applications`, `services`, `infra`, `domains`, `people`, `shares`,
`stacks`, `healthchecks`, `notes`.

- `auto_list` returns `{data: [...], total, page, per_page, total_pages}`.
  Without `all: true` you get one page of 50. Relations are shown as names;
  `full: true` returns the full objects. Filters: `status` and `environment`
  for applications and services.
- `auto_get` returns the entity's own fields plus relation arrays:
  - applications: `infra`, `services`, `domains`, `people` (with
    `contribution_type`: developer, maintainer, project_owner, stakeholder),
    `network_shares`, `stacks`, `healthchecks`, `notes`
  - services: `applications`, `infra`, `healthchecks`
  - infra: `ips`, `domain`, `applications`, `services`, `healthchecks`,
    `pointing_domains`
  - domains: `target_application_name` / `target_service_name` /
    `target_infra_name`, `applications`, `ips`
  - people, shares, stacks: `applications`
- `auto_search` returns `{applications: [...], services: [...], infra: [...], domains: [...], people: [...], network_shares: [...], stacks: [...], healthchecks: [...]}`,
  each hit `{id, name, description, entity_type}`.
- `auto_status` returns an array, worst first, of
  `{id, name, state, url, application_name, service_name, since, last_check, message, ping_ms, uptime_percent}`.
  `state` is one of down, pending, unknown (no data from Uptime Kuma),
  maintenance, up, disabled. `problems: true` means down, pending and
  unknown. `since` is limited to the last hour, which is all the server keeps.
- Common values: environment `dev`, `tst`, `qas`, `prd`; status `active`,
  `inactive`, `deprecated`, `archived`.

## How a site is served (proxy, load balancer, Nomad)

auto says which systems are involved; the deploy repos say how. When the
user has checkouts of the deploy repos, read the config and answer with the
concrete chain (domain, DNS, load balancer, upstream host and port, container
or Nomad job), citing files. Without them, answer from auto alone (steps 1
and 3) and say which repo holds the exact config.

1. In auto: `auto_get` on the domain (its target and `ips`) or on the
   application (its `domains` and `infra`). The load balancer is a service
   in auto; its `repository_url` is its config repo.
2. Local checkouts: `auto_deploy_repos` with `fetch: true`. It returns
   `{configured, repos, note}`: the user's checkouts with what they hold
   (`caddy`, `nomad`) and whether they are `behind` their remote. When
   `configured` is false, follow the `note`: it says whether to offer the
   setup (admins) or to answer from auto alone (everyone else).
   - Read the remote version, not a stale working tree, with Bash:
     `git -C <path> grep -n -e <fqdn> -e <name> origin/<branch>` and
     `git -C <path> show origin/<branch>:<file>`. Mention local changes
     (`dirty`) if they matter. Pull only when the user asks.
   - When the note says to offer the setup: `deploy_repos` lists local git
     checkouts of the Caddy proxy and load balancer config (`caddy-config`)
     and the Nomad jobs
     (`git@github.ugent.be:containers/nomad-jobs-ns-ghentcdh.git`). Ask for
     the paths of their checkouts (or offer to clone them), and offer to add
     them to `~/.config/auto/config.toml`:
     `deploy_repos = ["~/path/to/caddy-config", "~/path/to/nomad-jobs-ns-ghentcdh"]`.
     Someone without these checkouts can set `deploy_repos = []`, and is
     then not asked again.
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

## Changing data: always confirm first

Never change auto without the user's explicit yes for that exact change.

1. Look at the current state (`auto_get` / `auto_search`), so you do not
   create a duplicate.
2. Get the field names with `auto_schema`. It returns
   `{entity, create: [field], update: [field], list_filters: [field], relations: [{name, fields: [field]}]}`
   with `field = {name, type, required, default, enum}`. Do not guess names.
3. Show the user the change: call the tool with `dry_run: true` and show the
   request.
4. After the user agrees, call it again without `dry_run`.

| Change | Tool |
|---|---|
| Create | `auto_create` (entity, body) |
| Update (only the given fields) | `auto_update` (entity, id, body) |
| Link | `auto_link` (entity, id, relation, other, optional body) |
| Unlink | `auto_unlink` (entity, id, relation, other) |
| Delete | `auto_delete` (entity, id) |
| Sync to Uptime Kuma, Outline, refresh infra or domains | `auto_sync` |

- Links go from applications to infra, services, domains, people, shares and
  stacks, and from services to infra. The other direction gives an error that
  says how to swap.
- A domain needs exactly one target: `target_application_id`,
  `target_service_id` or `target_infra_id`. A healthcheck needs `domain_id`
  and one of `application_id` or `service_id`; after creating or changing
  one, call `auto_sync` (target `kuma`, the healthcheck id).

## Keeping auto in sync

When the user has changed infrastructure in this session, compare it with
auto (`auto_get` / `auto_search`). List the differences and propose the
changes, for example a new VM to create and link to the application, a
changed URL, or `status: deprecated` for a service that was switched off.
Wait for confirmation before you make them. For a full check of the
application or service in the current repo, use the `auto:audit` skill.

- Notes are for details the user wants in auto that do not fit its data
  model. Use them sparingly: what a field or a link can hold goes there, and
  repo documentation stays in the repo. Do not propose a note unless the user
  asks or a critical fact has no other place.
- People: one linked person in any role (project_owner, maintainer,
  developer, stakeholder) is enough. Do not ask for a maintainer when someone
  else is linked.

## Without the MCP tools

In a plain terminal, every tool has a CLI command with the same name and
arguments (`auto status --problems`, `auto get applications <name>`,
`auto here`, `auto deploy-repos --fetch`, ...). `auto --help` lists them.
