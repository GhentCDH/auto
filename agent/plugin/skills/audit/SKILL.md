---
name: audit
description: Check whether auto (the organization's IT inventory) is complete and up to date for the application or service developed in the current repository, and propose the changes. Covers healthchecks and their Uptime Kuma sync, linked servers/infra, services such as databases, domains, tech stack, people and important notes. Use when the user asks to check, audit, sync or update auto for "this app", "this repo" or a named application or service, or after changes to how the app is deployed or reached (deploy files, domains, servers, dependencies, health endpoints).
---

# Audit an application or service in auto

Goal: compare what the repository and the live systems say with what auto
records, then propose exact `auto` commands to close the gaps. Read-only until
the user confirms. Use the `auto` CLI as described in the `auto:auto` skill
(output is JSON when piped; do not run `--help` or read the OpenAPI document).

Work in this order and keep the number of commands small.

## 1. Identify the record

Run `auto here`. It returns `{repo: {root, remote, name}, matches: [...], deploy_repos: [{path, exists}]}`.

- `matches[].matched_by` is `repository_url` (certain) or `name` (a guess:
  confirm with the user).
- Several matches usually means one app per environment (prd, tst, ...).
  Audit all of them, or ask which one if the question was about one.
- No match: try `auto search <repo name>`. If you find it, propose
  `auto update <entity> <id> repository_url=<remote as https URL>` so the next
  audit matches directly. If it does not exist, offer to create it.

Then load each record: `auto get <entity> <id>`.

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

**Central deploy repos** (Caddy proxy config, Nomad jobs)
- Run `auto deploy-repos --fetch`, and follow "How a site is served" in the
  `auto:auto` skill: grep the remote version for the app name, the repo name
  and each domain (`git -C <path> grep -n -e <name> -e <fqdn> origin/<branch>`),
  then read only the files that hit.
- Caddy: every hop of every domain (load balancer, docker host) and the
  upstream host and port. A site file only in `sites-available/` is not live.
- Nomad: the app folder's `<env>.vars.yaml` per environment (domains, image,
  ports) and `job.nomad.hcl.j2` (datacenter, constraints, services such as
  databases).
- If no deploy repos are configured, say so in the report and offer to add
  the user's checkouts as `deploy_repos = [...]` in
  `~/.config/auto/config.toml`.

**Live checks**
- `auto status` (filter the JSON to this record's healthcheck ids).
- `auto dns <fqdn>` for each domain, to see where it points.
- Only if the user agrees, `curl -sS -o /dev/null -w '%{http_code}' <url>`
  on a candidate health endpoint.

## 3. Compare, area by area

| Area | In auto (`auto get`) | Gap when |
|---|---|---|
| Fields | `url`, `repository_url`, `environment`, `status` | empty, or different from the repo or deploy config |
| Domains | `domains` | a host name from the deploy or proxy config is not linked, or `auto dns` points elsewhere |
| Infra | `infra` | the deploy target (cluster, server, VM) is not linked, or a linked one is no longer used |
| Services | `services` | a dependency (database, cache, search, storage) is not linked |
| Healthchecks | `healthchecks` (`is_enabled`, `kuma_id`, `kuma_dirty`, `path`, `domain_fqdn`) | a public URL has no healthcheck; a health endpoint exists in the code but the check uses `/`; `kuma_id` is null or `kuma_dirty` is true (not synced to Kuma); `auto status` shows down or unknown |
| Stack | `stacks` | the main language or framework is not linked |
| People | `people` (`contribution_type`) | no `maintainer` and no `project_owner` |
| Notes | `notes` | rarely; see below |

Do not propose notes by default. Most facts belong in a field or a link, and
how-to documentation belongs in the repository or Outline. Propose a note
only when:

- the fact is really important for operations and auto has no field or link
  for it (for example, "data is lost on redeploy, there is no backup"), or
- the user asks to record something in auto that does not fit its model.

Keep a note short: one or two sentences. Never put secrets in a note: refer to
where they are stored.

## 4. Report, then fix with confirmation

Report as one table: area, what auto has, what you found (with the file or
command as evidence), and the proposed command. Mark each row `ok`,
`missing`, `outdated` or `check` (not sure). Put problems first. Keep `ok`
rows to one line.

Then ask which changes to make. For each accepted change:

1. Run it with `--dry-run` and show it.
2. Run it for real.

Commands you will need (field names come from `auto schema <entity>` when
unsure):

- New healthcheck:
  `auto create healthchecks name=<name> domain_id=<domain id> application_id=<id> path=/health`
  (or `service_id=`). Then `auto sync kuma <healthcheck id>` so Uptime Kuma
  gets it. Also sync after changing a check with `kuma_dirty` true.
- Link: `auto link applications <id> infra|services|domains|people|stacks <other> [key=value]`;
  services link only infra: `auto link services <id> infra <other>`.
  People take `contribution_type=maintainer` (or developer, project_owner,
  stakeholder).
- New domain: `auto create domains fqdn=<fqdn> target_application_id=<id>`
  (exactly one `target_*_id`), then link it.
- Note (only in the cases above): `auto create notes entity_type=application entity_id=<id> title=<title> content=<text>`.
- Fields: `auto update <entity> <id> url=<url> repository_url=<url>`.

End with a short summary of what changed and what is left open.
