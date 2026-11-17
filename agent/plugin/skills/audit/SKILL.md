---
name: audit
description: Check whether auto (the organization's IT inventory) is complete and up to date for the application or service developed in the current repository, and propose the changes. Covers healthchecks and their Uptime Kuma sync, linked servers/infra, services such as databases, domains, tech stack, people and important notes. Use when the user asks to check, audit, sync or update auto for "this app", "this repo" or a named application or service, or after changes to how the app is deployed or reached (deploy files, domains, servers, dependencies, health endpoints).
---

# Audit an application or service in auto

Goal: compare what the repository, the deploy repos and the live systems say
with what auto records, then propose exact changes to close the gaps.
Read-only until the user confirms. Use the `auto_*` MCP tools as described in
the `auto:auto` skill. Keep the number of calls small.

## 1. Identify the record

Call `auto_here` with the project's path. It returns
`{repo: {root, remote, name}, matches: [...], deploy_repos: [...]}`.

- `matches[].matched_by` is `repository_url` (certain) or `name` (a guess:
  confirm with the user).
- Several matches usually means one record per environment (prd, tst, ...).
  Audit all of them, or ask which one if the question was about one.
- No match: try `auto_search` with the repo name. If you find it, propose
  setting its `repository_url` (as an https URL) with `auto_update`, so the
  next audit matches directly. If it does not exist, offer to create it.

Then load each record with `auto_get`.

## 2. Gather evidence

Look only where it matters; do not read the whole repo.

**This repository**
- Deploy files: `*.nomad`, `*.nomad.hcl`, `docker-compose*.yml`,
  `compose.y*ml`, `Dockerfile*`, `Caddyfile`, `.github/workflows/*`,
  `deploy/`, `k8s/`, `helm/`, `*.tf`, `fly.toml`.
- Domains: host names in those files, in `README*` and in `.env.example`.
- Dependencies: images in compose files (postgres, mysql, mariadb, redis,
  elasticsearch, opensearch, minio, rabbitmq, ...), and environment
  variables like `DATABASE_URL`, `REDIS_URL`, `*_HOST`.
- Health endpoints: routes such as `/health`, `/healthz`, `/api/health`,
  `/status`, `/ping` in the code. Grep for them.
- Stack: `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements.txt`,
  `composer.json`, `go.mod`, `Gemfile`, `pom.xml`: the main language and
  framework.
- Maintainers: `git shortlog -sne --since=1.year HEAD | head -5`, only as a
  hint for the user.

**Deploy repos** (Caddy proxy config, Nomad jobs)
- Call `auto_deploy_repos` with `fetch: true`, and follow "How a site is
  served" in the `auto:auto` skill: grep the remote version for the app
  name, the repo name and each domain
  (`git -C <path> grep -n -e <name> -e <fqdn> origin/<branch>`), then read
  only the files that hit.
- Caddy: every hop of every domain (load balancer, docker host) and the
  upstream host and port. A site file only in `sites-available/` is not live.
- Nomad: the app folder's `<env>.vars.yaml` per environment (domains, image,
  ports) and `job.nomad.hcl.j2` (datacenter, constraints, services such as
  databases).
- No deploy repos configured: say so in the report and offer to add the
  user's checkouts as `deploy_repos = [...]` in `~/.config/auto/config.toml`.

**Live checks**
- `auto_status` (keep this record's healthcheck ids).
- `auto_dns` for each domain, to see where it points.
- Only if the user agrees, `curl -sS -o /dev/null -w '%{http_code}' <url>`
  on a candidate health endpoint.

## 3. Compare, area by area

| Area | In auto (`auto_get`) | Gap when |
|---|---|---|
| Fields | `url`, `repository_url`, `environment`, `status` | empty, or different from the repo or deploy config |
| Domains | `domains` | a host name from the deploy or proxy config is not linked, or `auto_dns` points elsewhere |
| Infra | `infra` | the deploy target (cluster, server, VM) is not linked, or a linked one is no longer used |
| Services | `services` | a dependency (database, cache, search, storage) is not linked |
| Healthchecks | `healthchecks` (`is_enabled`, `kuma_id`, `kuma_dirty`, `path`, `domain_fqdn`) | a public URL has no healthcheck; a health endpoint exists in the code but the check uses `/`; `kuma_id` is null or `kuma_dirty` is true (not synced to Kuma); `auto_status` shows down or unknown |
| Stack | `stacks` | the main language or framework is not linked |
| People | `people` (`contribution_type`) | no `maintainer` |
| Notes | `notes` | important operational knowledge is missing (see below) |

Notes worth proposing: how to deploy or roll back, backups and restore,
scheduled jobs, data migrations, external dependencies (APIs, SSO, mail),
unusual ports or networking. Never put secrets in a note: refer to where they
are stored.

## 4. Report, then fix with confirmation

Report as one table: area, what auto has, what you found (with the file or
tool call as evidence), and the proposed change. Mark each row `ok`,
`missing`, `outdated` or `check` (not sure). Put problems first. Keep `ok`
rows to one line.

Then ask which changes to make. For each accepted change, call the tool with
`dry_run: true` and show it, then call it for real. Use `auto_schema` when
unsure about a field name.

- New healthcheck: `auto_create`, entity `healthchecks`, body
  `{name, domain_id, application_id (or service_id), path}`. Then
  `auto_sync` with target `kuma` and the new healthcheck id, so Uptime Kuma
  gets it. Also sync a check whose `kuma_dirty` is true.
- Link: `auto_link` from applications to `infra`, `services`, `domains`,
  `people`, `stacks`, `shares`; from services only to `infra`. People take
  body `{contribution_type: maintainer}` (or developer, project_owner,
  stakeholder).
- New domain: `auto_create`, entity `domains`, body
  `{fqdn, target_application_id}` (exactly one `target_*_id`), then link it.
- Note: `auto_create`, entity `notes`, body
  `{entity_type: application, entity_id, title, content, note_type: documentation}`.
- Fields: `auto_update` with `{url, repository_url, ...}`.

End with a short summary of what changed and what is left open.
