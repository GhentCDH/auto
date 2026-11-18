//! `auto mcp`: an MCP server on stdio that exposes the same generic
//! operations as the CLI. Tool schemas are built from `EntityKind::ALL`, and
//! field details come from the server via `auto_schema`, so the tool list
//! does not change when the backend gains fields.

use std::path::PathBuf;
use std::sync::Arc;

use color_eyre::eyre::{Result, eyre};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::{Value, json};
use tokio::sync::OnceCell;

use crate::api::{ApiClient, EntityKind};
use crate::cli::PROBLEM_STATES;
use crate::config::{Config, DEPLOY_REPOS_HELP};
use crate::here;
use crate::ops::{Action, ListQuery, Ops};
use crate::view::row_label;

const INSTRUCTIONS: &str = "\
auto is the organization's inventory of IT infrastructure: applications, \
services, infra (servers, VMs, clusters), domains and DNS, people and who \
maintains what, network shares, technology stacks, healthchecks and notes.

Use it to answer questions about what runs where, who owns or maintains \
something, which domains point to which machines, and what is monitored.

Shortcuts:
- \"What is down?\": auto_status with problems=true. One call; never run \
  healthchecks one by one.
- Where an app runs, who maintains it, its domains: auto_get on the \
  application. What runs on a server and which domains point to it: auto_get \
  on the infra.
- Which record the current git repo is: auto_here with the project path.
- How a site is proxied or deployed: auto says which systems are involved; \
  auto_deploy_repos (fetch=true) lists the user's checkouts of the Caddy and \
  Nomad config repos. If you can run commands, read the remote version \
  (git -C <path> grep -n <fqdn> origin/<branch>) and answer with the concrete \
  chain, citing files. Without checkouts, answer from auto alone.

Rules:
- Read before you write. Find the entity with auto_search or auto_list, and \
  inspect it with auto_get.
- Before any create, update, delete, link or unlink: call auto_schema for the \
  field names, show the user the exact change, and get their confirmation. \
  Use dry_run=true to show the request without sending it.
- Ids can be full ids, id prefixes, or exact names.
- When the user has just changed infrastructure (deployed something, added a \
  VM, changed DNS, decommissioned a service), offer to update auto to match.
- A write that fails with \"not allowed\" means the user can only read: stop \
  proposing changes to auto for the rest of the session.";

pub async fn serve(client: ApiClient) -> Result<()> {
    let server = AutoMcp {
        ops: Ops::new(client),
        authed: OnceCell::new(),
    };
    let running = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| eyre!("MCP startup failed: {e}"))?;
    running.waiting().await?;
    Ok(())
}

struct AutoMcp {
    ops: Ops,
    authed: OnceCell<()>,
}

impl ServerHandler for AutoMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("auto", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let args = Args(request.arguments.unwrap_or_default());
        let result = match self
            .authed
            .get_or_try_init(|| self.ops.client.require_auth())
            .await
        {
            Ok(()) => self.dispatch(&request.name, &args).await,
            Err(error) => Err(error),
        };
        Ok(match result {
            Ok(Value::String(text)) => CallToolResult::success(vec![ContentBlock::text(text)]),
            Ok(value) => CallToolResult::success(vec![ContentBlock::text(value.to_string())]),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(format!("{error:#}"))]),
        }
        .into())
    }
}

impl AutoMcp {
    async fn dispatch(&self, tool: &str, args: &Args) -> Result<Value> {
        let ops = &self.ops;
        let dry_run = args.flag("dry_run");
        match tool {
            "auto_list" => {
                let query = ListQuery {
                    search: args.opt_str("search"),
                    filters: args
                        .0
                        .get("filters")
                        .and_then(Value::as_object)
                        .into_iter()
                        .flatten()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.as_str().map_or_else(|| v.to_string(), str::to_string),
                            )
                        })
                        .collect(),
                    page: args.0.get("page").and_then(Value::as_u64).map(|p| p as u32),
                    per_page: args
                        .0
                        .get("per_page")
                        .and_then(Value::as_u64)
                        .map(|p| p as u32),
                    all: args.flag("all"),
                };
                let mut response = ops.list(args.entity("entity")?, &query).await?;
                if !args.flag("full")
                    && let Some(Value::Array(rows)) = response.get_mut("data")
                {
                    rows.iter_mut().for_each(summarize);
                }
                Ok(response)
            }
            "auto_get" => ops.get(args.entity("entity")?, args.str("id")?).await,
            "auto_search" => ops.search(args.str("query")?).await,
            "auto_status" => {
                let states: Vec<String> = match args.flag("problems") {
                    true => PROBLEM_STATES.map(String::from).to_vec(),
                    false => args
                        .0
                        .get("state")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|s| s.as_str().map(str::to_string))
                        .collect(),
                };
                ops.status(&states).await
            }
            "auto_schema" => Ok(serde_json::to_value(
                ops.schema(args.entity("entity")?).await?,
            )?),
            "auto_overview" => ops
                .overview(args.entity("entity")?, args.str("id")?)
                .await
                .map(Value::String),
            "auto_create" => {
                let request = ops.create(args.entity("entity")?, args.body()?);
                ops.send(&request, dry_run).await
            }
            "auto_update" => {
                let request = ops
                    .update(args.entity("entity")?, args.str("id")?, args.body()?)
                    .await?;
                ops.send(&request, dry_run).await
            }
            "auto_delete" => {
                let kind = args.entity("entity")?;
                let (id, value) = ops.resolve(kind, args.str("id")?).await?;
                let request = ops.delete(kind, &id).await?;
                let result = ops.send(&request, dry_run).await?;
                Ok(match dry_run {
                    true => result,
                    false => {
                        json!({ "deleted": { "entity": kind.path(), "id": id, "name": row_label(&value) } })
                    }
                })
            }
            "auto_link" | "auto_unlink" => {
                let unlink = tool == "auto_unlink";
                let body = args.0.get("body").cloned();
                let request = ops
                    .link(
                        args.entity("entity")?,
                        args.str("id")?,
                        args.entity("relation")?,
                        args.str("other")?,
                        body,
                        unlink,
                    )
                    .await?;
                let result = ops.send(&request, dry_run).await?;
                Ok(match (dry_run, unlink) {
                    (true, _) => result,
                    (false, false) => json!({ "linked": request.path }),
                    (false, true) => json!({ "unlinked": request.path }),
                })
            }
            "auto_dns" => {
                ops.action(Action::Dns {
                    domain: args.str("domain")?.into(),
                })
                .await
            }
            "auto_run_healthcheck" => {
                ops.action(Action::RunHealthcheck {
                    id: args.str("id")?.into(),
                })
                .await
            }
            "auto_sync" => {
                let id = args.opt_str("id");
                let action = match args.str("target")? {
                    "infra" => Action::SyncInfra { id },
                    "kuma" => Action::SyncKuma { id },
                    "domains" => Action::SyncDomains,
                    "outline" => Action::SyncOutline {
                        kind: args.entity("entity")?,
                        id: id.ok_or_else(|| eyre!("outline sync needs `id`"))?,
                    },
                    other => return Err(eyre!("unknown sync target `{other}`")),
                };
                ops.action(action).await
            }
            "auto_here" => {
                let path = args.opt_str("path").map(PathBuf::from);
                let path = match path {
                    Some(path) => path,
                    None => std::env::current_dir()?,
                };
                let repo = here::repo_at(&path)
                    .ok_or_else(|| eyre!("{} is not inside a git repository", path.display()))?;
                let deploy_repos = Config::deploy_repo_paths()?;
                let found = here::find(ops, &repo, &deploy_repos).await?;
                Ok(serde_json::to_value(found)?)
            }
            "auto_deploy_repos" => {
                const FROM_AUTO: &str = "Answer from auto alone (the domain, its target \
                    infra, the load balancer service) and say which repo holds the exact \
                    config.";
                let note = match Config::deploy_repo_setting()? {
                    Some(paths) if !paths.is_empty() => {
                        let repos = here::deploy_repos(&paths, args.flag("fetch")).await;
                        return Ok(json!({ "configured": true, "repos": repos }));
                    }
                    Some(_) => format!(
                        "The user set deploy_repos = [] (no checkouts). {FROM_AUTO} Do not \
                         ask about deploy repos."
                    ),
                    None => match here::user_role(&ops.client).await {
                        Some(here::Role::Admin) => format!(
                            "Not set up yet. {DEPLOY_REPOS_HELP} Explain this to the user \
                             once, ask for the paths of their checkouts, and offer to add \
                             them. Until then: {FROM_AUTO}"
                        ),
                        _ => format!(
                            "The user has no checkouts of the deploy repos. {FROM_AUTO} Do \
                             not ask the user to set them up."
                        ),
                    },
                };
                Ok(json!({ "configured": false, "repos": [], "note": note }))
            }
            other => Err(eyre!("unknown tool `{other}`")),
        }
    }
}

/// Typed access to tool arguments with readable errors.
struct Args(JsonObject);

impl Args {
    fn str(&self, key: &str) -> Result<&str> {
        self.0
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| eyre!("missing string argument `{key}`"))
    }

    fn opt_str(&self, key: &str) -> Option<String> {
        self.0.get(key).and_then(Value::as_str).map(str::to_string)
    }

    fn flag(&self, key: &str) -> bool {
        self.0.get(key).and_then(Value::as_bool).unwrap_or(false)
    }

    fn entity(&self, key: &str) -> Result<EntityKind> {
        self.str(key)?.parse().map_err(|e: String| eyre!(e))
    }

    fn body(&self) -> Result<Value> {
        match self.0.get("body") {
            Some(Value::Object(body)) => Ok(Value::Object(body.clone())),
            _ => Err(eyre!(
                "`body` must be a JSON object; call auto_schema for the fields"
            )),
        }
    }
}

/// Replace nested relation arrays with their labels, so a list stays small.
/// `auto_get` returns the full entity.
fn summarize(row: &mut Value) {
    if let Value::Object(map) = row {
        map.values_mut()
            .filter(|value| {
                value
                    .as_array()
                    .is_some_and(|a| a.iter().all(Value::is_object))
            })
            .for_each(|value| {
                let labels = value
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|entry| Value::String(row_label(entry).to_string()))
                    .collect();
                *value = Value::Array(labels);
            });
    }
}

fn tools() -> Vec<Tool> {
    let entities: Vec<&str> = EntityKind::ALL.iter().map(|k| k.path()).collect();
    let entity = json!({ "type": "string", "enum": entities });
    let id = json!({ "type": "string", "description": "Full id, id prefix, or exact name" });
    let dry_run =
        json!({ "type": "boolean", "description": "Return the request without sending it" });
    let read = || ToolAnnotations::new().read_only(true).open_world(false);
    let write = || ToolAnnotations::new().read_only(false).open_world(false);

    vec![
        tool(
            "auto_list",
            "List entities of one kind. Relations are shown as names; use auto_get for \
             full details. `filters` takes list filters such as status or environment \
             (see auto_schema).",
            json!({
                "entity": entity,
                "search": { "type": "string" },
                "filters": { "type": "object", "additionalProperties": { "type": "string" } },
                "page": { "type": "integer", "minimum": 1 },
                "per_page": { "type": "integer", "minimum": 1, "maximum": 100 },
                "all": { "type": "boolean", "description": "Fetch every page" },
                "full": { "type": "boolean", "description": "Return full relation objects" },
            }),
            &["entity"],
            read(),
        ),
        tool(
            "auto_get",
            "Get one entity with all its relations.",
            json!({ "entity": entity, "id": id }),
            &["entity", "id"],
            read(),
        ),
        tool(
            "auto_search",
            "Search all entity kinds at once by name or description.",
            json!({ "query": { "type": "string" } }),
            &["query"],
            read(),
        ),
        tool(
            "auto_status",
            "Live state of every healthcheck (website/service monitor in Uptime Kuma), \
             worst first: state (down, pending, unknown, maintenance, up, disabled), \
             since, message, url and the application or service it belongs to. \
             Use this for \"what is down?\"; do not run healthchecks one by one.",
            json!({
                "problems": { "type": "boolean", "description": "Only down, pending and unknown" },
                "state": {
                    "type": "array",
                    "items": { "type": "string", "enum": ["down", "pending", "unknown", "maintenance", "up", "disabled"] },
                },
            }),
            &[],
            read(),
        ),
        tool(
            "auto_schema",
            "Fields for create and update, list filters, and link relations (with their \
             fields) of an entity kind. Call this before writing.",
            json!({ "entity": entity }),
            &["entity"],
            read(),
        ),
        tool(
            "auto_overview",
            "Markdown overview of an application or service.",
            json!({ "entity": { "type": "string", "enum": ["applications", "services"] }, "id": id }),
            &["entity", "id"],
            read(),
        ),
        tool(
            "auto_create",
            "Create an entity. Omitted fields get the server defaults. Confirm with the \
             user first.",
            json!({ "entity": entity, "body": { "type": "object" }, "dry_run": dry_run }),
            &["entity", "body"],
            write(),
        ),
        tool(
            "auto_update",
            "Update an entity. Only the fields in `body` change. Confirm with the user first.",
            json!({ "entity": entity, "id": id, "body": { "type": "object" }, "dry_run": dry_run }),
            &["entity", "id", "body"],
            write().idempotent(true),
        ),
        tool(
            "auto_delete",
            "Delete an entity. Confirm with the user first.",
            json!({ "entity": entity, "id": id, "dry_run": dry_run }),
            &["entity", "id"],
            write().destructive(true),
        ),
        tool(
            "auto_link",
            "Link two entities, e.g. entity=applications relation=people. `body` holds \
             link fields (see relations in auto_schema). Confirm with the user first.",
            json!({
                "entity": entity, "id": id, "relation": entity, "other": id,
                "body": { "type": "object" }, "dry_run": dry_run,
            }),
            &["entity", "id", "relation", "other"],
            write().idempotent(true),
        ),
        tool(
            "auto_unlink",
            "Remove a link between two entities. Confirm with the user first.",
            json!({ "entity": entity, "id": id, "relation": entity, "other": id, "dry_run": dry_run }),
            &["entity", "id", "relation", "other"],
            write().destructive(true),
        ),
        tool(
            "auto_dns",
            "Live DNS records of a domain.",
            json!({ "domain": id }),
            &["domain"],
            ToolAnnotations::new().read_only(true).open_world(true),
        ),
        tool(
            "auto_run_healthcheck",
            "Run a healthcheck now and return the result.",
            json!({ "id": id }),
            &["id"],
            ToolAnnotations::new().read_only(true).open_world(true),
        ),
        tool(
            "auto_sync",
            "Trigger a server-side sync: infra status, Uptime Kuma monitors, domain data, \
             or the Outline page of an application or service (needs entity and id).",
            json!({
                "target": { "type": "string", "enum": ["infra", "kuma", "domains", "outline"] },
                "entity": { "type": "string", "enum": ["applications", "services"] },
                "id": id,
            }),
            &["target"],
            write().open_world(true),
        ),
        tool(
            "auto_here",
            "Which auto application or service a git repository is (matched by \
             repository_url, else by name: confirm name matches with the user), plus \
             the user's deploy repos. Pass the repo path of the current project.",
            json!({ "path": { "type": "string", "description": "Path inside the git repository" } }),
            &["path"],
            read(),
        ),
        tool(
            "auto_deploy_repos",
            "The user's local checkouts of the central deploy repos (Caddy proxy \
             config, Nomad jobs): {configured, repos, note}. Each repo has its path, \
             what it holds (caddy, nomad, compose), branch, local changes and how far \
             behind its remote it is. With fetch=true it runs `git fetch` first. Read \
             the files from origin/<branch> with git, not from the working tree. When \
             configured is false, follow the note.",
            json!({ "fetch": { "type": "boolean" } }),
            &[],
            ToolAnnotations::new().read_only(true).open_world(true),
        ),
    ]
}

fn tool(
    name: &'static str,
    description: &'static str,
    properties: Value,
    required: &[&str],
    annotations: ToolAnnotations,
) -> Tool {
    let schema = json!({ "type": "object", "properties": properties, "required": required });
    let schema: JsonObject = match schema {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    };
    Tool::new(name, description, Arc::new(schema)).with_annotations(annotations)
}

#[cfg(test)]
mod tests {
    use super::tools;

    /// The skill is how Claude learns when to use which tool, so every tool
    /// must be in it.
    #[test]
    fn skill_mentions_every_tool() {
        const SKILL: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../agent/plugin/skills/auto/SKILL.md"
        ));
        let missing: Vec<String> = tools()
            .iter()
            .map(|tool| tool.name.to_string())
            .filter(|name| !SKILL.contains(&format!("`{name}`")))
            .collect();
        assert!(
            missing.is_empty(),
            "agent/plugin/skills/auto/SKILL.md does not mention: {missing:?}"
        );
    }
}
