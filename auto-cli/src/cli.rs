//! Command line surface. Every subcommand is a thin wrapper around `ops`.

use std::io::{self, IsTerminal, Read};

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{Result, eyre};
use reqwest::Method;
use serde_json::{Map, Value, json};

use crate::api::{ApiClient, EntityKind};
use crate::config;
use crate::openapi::{self, Field};
use crate::ops::{Action, ListQuery, Ops, Request};
use crate::output::{self, Format};
use crate::session_store;
use crate::view::row_label;

/// Healthcheck states that need attention (`auto status --problems`).
pub const PROBLEM_STATES: [&str; 3] = ["down", "pending", "unknown"];

/// Client for the auto admin API. Without a subcommand it opens the TUI.
#[derive(Debug, Parser)]
#[command(name = "auto", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct Global {
    /// Server URL (overrides AUTO_URL and the config file).
    #[arg(long, global = true)]
    pub url: Option<String>,
    /// Output JSON (same as --format json).
    #[arg(long, global = true)]
    pub json: bool,
    /// Output format. Defaults to table on a terminal, JSON otherwise.
    #[arg(long, global = true, value_enum)]
    pub format: Option<Format>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sign in through the browser (password or SSO) and store the session.
    Login,
    /// Sign out and forget the stored session.
    Logout,
    /// Show the signed-in user and role.
    Whoami,
    /// Manage personal API tokens (for scripts, agents and MCP).
    #[command(subcommand)]
    Token(TokenCommand),
    /// List entities.
    #[command(visible_alias = "ls")]
    List {
        entity: EntityKind,
        /// Text search on name and other fields.
        #[arg(short, long)]
        search: Option<String>,
        /// Extra filter, e.g. `-F status=active -F environment=prd`.
        /// See `auto schema <entity>` for the filters.
        #[arg(short = 'F', long = "filter", value_name = "KEY=VALUE")]
        filters: Vec<String>,
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        per_page: Option<u32>,
        /// Fetch every page.
        #[arg(long)]
        all: bool,
    },
    /// Show one entity with its relations, by id, id prefix or exact name.
    #[command(visible_alias = "show")]
    Get { entity: EntityKind, id: String },
    /// Search all entities.
    Search {
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,
    },
    /// Live state of every healthcheck (from Uptime Kuma), worst first.
    Status {
        /// Only checks that need attention: down, pending or unknown.
        #[arg(short, long)]
        problems: bool,
        /// Only these states (down, pending, unknown, maintenance, up, disabled).
        #[arg(short, long, value_delimiter = ',', conflicts_with = "problems")]
        state: Vec<String>,
    },
    /// Show the fields, filters and link relations of an entity, read from
    /// the server's OpenAPI document. Without an entity, list the entities.
    Schema { entity: Option<EntityKind> },
    /// Create an entity from `key=value` pairs and/or a JSON body.
    Create {
        entity: EntityKind,
        #[command(flatten)]
        body: BodyArgs,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Update fields of an entity. Only the given fields change.
    Update {
        entity: EntityKind,
        id: String,
        #[command(flatten)]
        body: BodyArgs,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Delete an entity.
    #[command(visible_alias = "rm")]
    Delete {
        entity: EntityKind,
        id: String,
        /// Do not ask for confirmation.
        #[arg(short, long)]
        yes: bool,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Link two entities, e.g. `auto link applications wiki people alice contribution_type=maintainer`.
    Link {
        entity: EntityKind,
        id: String,
        relation: EntityKind,
        other: String,
        #[command(flatten)]
        body: BodyArgs,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Remove a link between two entities.
    Unlink {
        entity: EntityKind,
        id: String,
        relation: EntityKind,
        other: String,
        #[command(flatten)]
        write: WriteArgs,
    },
    /// Markdown overview of an application or service.
    Overview { entity: EntityKind, id: String },
    /// Live DNS records of a domain.
    Dns { domain: String },
    /// Run a healthcheck now and show the result.
    Check { healthcheck: String },
    /// Trigger a server-side sync.
    #[command(subcommand)]
    Sync(SyncCommand),
    /// Send a raw request to any API endpoint, e.g. `auto api GET /infra/loads`.
    Api {
        /// HTTP method (GET, POST, PUT, DELETE, ...), any case.
        #[arg(value_parser = parse_method)]
        method: Method,
        /// Path under /api (the /api prefix is optional).
        path: String,
        /// JSON body (`-` reads stdin).
        #[arg(short, long)]
        data: Option<String>,
        /// Query parameter, repeatable.
        #[arg(short, long = "query", value_name = "KEY=VALUE")]
        query: Vec<String>,
    },
    /// Which auto application or service this git repository is (by
    /// repository_url, else by name), plus the configured deploy repos.
    Here,
    /// Local checkouts of the central deploy repos (`deploy_repos` in the
    /// config): what they hold (caddy, nomad, compose) and how fresh they are.
    DeployRepos {
        /// Run `git fetch` in each first, so `behind` is current.
        #[arg(long)]
        fetch: bool,
    },
    /// Run an MCP server on stdio, for Claude Desktop and other agents.
    Mcp,
    /// Claude Code plugin hook (reads the hook input on stdin).
    #[command(hide = true)]
    Hook {
        #[arg(value_parser = ["session-start", "post-edit"])]
        event: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum TokenCommand {
    /// Create a token. The secret is shown once.
    Create {
        name: String,
        /// Lifetime in days; omit for a token that does not expire.
        #[arg(long)]
        expires_in_days: Option<i64>,
        /// Store the token (and the server URL) in ~/.config/auto/config.toml.
        #[arg(long)]
        save: bool,
    },
    /// List your tokens.
    List,
    /// Revoke a token by id.
    Revoke { id: String },
}

#[derive(Debug, Subcommand)]
pub enum SyncCommand {
    /// Refresh infra status (all, or one).
    Infra { id: Option<String> },
    /// Push healthchecks to Uptime Kuma (all, or one).
    Kuma { id: Option<String> },
    /// Refresh domain registration data.
    Domains,
    /// Push the overview of an application or service to Outline.
    Outline { entity: EntityKind, id: String },
}

#[derive(Debug, Args)]
pub struct BodyArgs {
    /// Fields as `key=value` (typed from the schema) or `key:=<json>`.
    #[arg(value_name = "KEY=VALUE")]
    pairs: Vec<String>,
    /// JSON body, merged under the pairs (`-` reads stdin).
    #[arg(short, long)]
    data: Option<String>,
}

#[derive(Debug, Args)]
pub struct WriteArgs {
    /// Print the request without sending it.
    #[arg(long)]
    dry_run: bool,
}

pub async fn run(command: Command, global: &Global) -> Result<()> {
    let format = Format::resolve(global.format, global.json);

    // Commands that work before any server is configured or signed in.
    match &command {
        Command::Login => return login(global).await,
        Command::Logout => return logout(global).await,
        Command::Mcp => return crate::mcp::serve(client(global)?).await,
        Command::Hook { event } => {
            crate::here::run_hook(event, global.url.as_deref()).await;
            return Ok(());
        }
        Command::DeployRepos { fetch } => {
            let paths = config::Config::deploy_repo_paths()?;
            if paths.is_empty() {
                return Err(eyre!(
                    "no deploy repos configured. {}",
                    config::DEPLOY_REPOS_HELP
                ));
            }
            let repos = crate::here::deploy_repos(&paths, *fetch).await;
            output::deploy_repos(&repos, format)?;
            return Ok(());
        }
        _ => {}
    }

    let client = client(global)?;
    client.require_auth().await?;
    crate::update_check::notify_if_outdated(&client).await;
    let ops = Ops::new(client);

    match command {
        Command::Login
        | Command::Logout
        | Command::Mcp
        | Command::Hook { .. }
        | Command::DeployRepos { .. } => {
            unreachable!("handled above")
        }
        Command::Here => {
            let config = config::Config::load(global.url.as_deref())?;
            let repo = crate::here::repo_at(&std::env::current_dir()?)
                .ok_or_else(|| eyre!("not inside a git repository"))?;
            let here = crate::here::find(&ops, &repo, &config.deploy_repos).await?;
            output::here(&here, format)?;
        }
        Command::Whoami => {
            let me = ops.client.call(Method::GET, "/auth/me", &[], None).await?;
            output::value(&me, format)?;
        }
        Command::Token(command) => token(&ops, command, format).await?,
        Command::List {
            entity,
            search,
            filters,
            page,
            per_page,
            all,
        } => {
            let query = ListQuery {
                search,
                filters: filters
                    .iter()
                    .map(|f| split_pair(f))
                    .collect::<Result<_>>()?,
                page,
                per_page,
                all,
            };
            output::list(entity, &ops.list(entity, &query).await?, format)?;
        }
        Command::Get { entity, id } => output::detail(&ops.get(entity, &id).await?, format)?,
        Command::Search { query } => output::search(&ops.search(&query.join(" ")).await?, format)?,
        Command::Status { problems, state } => {
            let states = match problems {
                true => PROBLEM_STATES.map(String::from).to_vec(),
                false => state,
            };
            output::status(&ops.status(&states).await?, format)?;
        }
        Command::Schema { entity: None } => {
            let entities: Vec<&str> = EntityKind::ALL.iter().map(|k| k.path()).collect();
            match format {
                Format::Json => output::json(&entities)?,
                _ => println!("{}", entities.join("\n")),
            }
        }
        Command::Schema {
            entity: Some(entity),
        } => output::schema(&ops.schema(entity).await?, format)?,
        Command::Create {
            entity,
            body,
            write,
        } => {
            let fields = ops.schema(entity).await?.create;
            let request = ops.create(entity, body.build(&fields)?);
            send(
                &ops,
                &request,
                write.dry_run,
                format,
                &format!("created {entity}"),
            )
            .await?;
        }
        Command::Update {
            entity,
            id,
            body,
            write,
        } => {
            let fields = ops.schema(entity).await?.update;
            let request = ops.update(entity, &id, body.build(&fields)?).await?;
            send(
                &ops,
                &request,
                write.dry_run,
                format,
                &format!("updated {entity}"),
            )
            .await?;
        }
        Command::Delete {
            entity,
            id,
            yes,
            write,
        } => {
            let (full_id, value) = ops.resolve(entity, &id).await?;
            if !yes && !write.dry_run {
                confirm(&format!(
                    "delete {entity} `{}` ({full_id})?",
                    row_label(&value)
                ))?;
            }
            let request = ops.delete(entity, &full_id).await?;
            send(
                &ops,
                &request,
                write.dry_run,
                format,
                &format!("deleted {entity}"),
            )
            .await?;
        }
        Command::Link {
            entity,
            id,
            relation,
            other,
            body,
            write,
        } => {
            let fields = ops
                .schema(entity)
                .await?
                .relations
                .into_iter()
                .find(|r| r.name == relation.path())
                .map(|r| r.fields)
                .unwrap_or_default();
            let body = body.build(&fields)?;
            let request = ops
                .link(entity, &id, relation, &other, Some(body), false)
                .await?;
            send(
                &ops,
                &request,
                write.dry_run,
                format,
                &format!("linked {relation}"),
            )
            .await?;
        }
        Command::Unlink {
            entity,
            id,
            relation,
            other,
            write,
        } => {
            let request = ops.link(entity, &id, relation, &other, None, true).await?;
            send(
                &ops,
                &request,
                write.dry_run,
                format,
                &format!("unlinked {relation}"),
            )
            .await?;
        }
        Command::Overview { entity, id } => println!("{}", ops.overview(entity, &id).await?),
        Command::Dns { domain } => {
            output::value(&ops.action(Action::Dns { domain }).await?, format)?
        }
        Command::Check { healthcheck } => {
            let result = ops
                .action(Action::RunHealthcheck { id: healthcheck })
                .await?;
            output::value(&result, format)?;
        }
        Command::Sync(command) => {
            let action = match command {
                SyncCommand::Infra { id } => Action::SyncInfra { id },
                SyncCommand::Kuma { id } => Action::SyncKuma { id },
                SyncCommand::Domains => Action::SyncDomains,
                SyncCommand::Outline { entity, id } => Action::SyncOutline { kind: entity, id },
            };
            let result = ops.action(action).await?;
            output::value(&result, format)?;
            output::note("sync done");
        }
        Command::Api {
            method,
            path,
            data,
            query,
        } => {
            let body = data.map(|d| read_json(&d)).transpose()?;
            let query: Vec<(String, String)> =
                query.iter().map(|q| split_pair(q)).collect::<Result<_>>()?;
            let path = format!("/{}", path.trim_start_matches('/'));
            let result = ops
                .client
                .call(method, &path, &query, body.as_ref())
                .await?;
            output::value(&result, format)?;
        }
    }
    Ok(())
}

fn client(global: &Global) -> Result<ApiClient> {
    ApiClient::new(&config::Config::load(global.url.as_deref())?)
}

impl BodyArgs {
    /// Merge the JSON body (if any) with the typed `key=value` pairs.
    fn build(self, fields: &[Field]) -> Result<Value> {
        let mut body: Map<String, Value> = match self.data.as_deref().map(read_json).transpose()? {
            Some(Value::Object(map)) => map,
            Some(_) => return Err(eyre!("--data must be a JSON object")),
            None => Map::new(),
        };
        body.extend(openapi::body_from_pairs(fields, &self.pairs)?);
        Ok(Value::Object(body))
    }
}

async fn send(
    ops: &Ops,
    request: &Request,
    dry_run: bool,
    format: Format,
    done: &str,
) -> Result<()> {
    if dry_run {
        output::note(&format!("dry run, not sent: {request}"));
        if format == Format::Json {
            output::json(request)?;
        }
        return Ok(());
    }
    let result = ops.send(request, false).await?;
    let is_entity = result.get("id").is_some();
    let label = match is_entity {
        true => format!(" `{}`", row_label(&result)),
        false => String::new(),
    };
    output::note(&format!("{done}{label}"));
    // People get the changed entity; for body-less writes the note is enough.
    if is_entity || format == Format::Json {
        output::value(&result, format)?;
    }
    Ok(())
}

fn confirm(question: &str) -> Result<()> {
    if !io::stdin().is_terminal() {
        return Err(eyre!("{question} pass --yes to confirm without a terminal"));
    }
    eprint!("{question} [y/N] ");
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        Err(eyre!("cancelled"))
    }
}

fn parse_method(method: &str) -> std::result::Result<Method, String> {
    Method::from_bytes(method.to_ascii_uppercase().as_bytes()).map_err(|e| e.to_string())
}

fn split_pair(pair: &str) -> Result<(String, String)> {
    pair.split_once('=')
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .ok_or_else(|| eyre!("expected KEY=VALUE, got `{pair}`"))
}

/// Parse a JSON argument, or read it from stdin when it is `-`.
fn read_json(data: &str) -> Result<Value> {
    let text = match data {
        "-" => {
            let mut text = String::new();
            io::stdin().read_to_string(&mut text)?;
            text
        }
        data => data.to_string(),
    };
    serde_json::from_str(&text).map_err(|e| eyre!("invalid JSON body: {e}"))
}

async fn login(global: &Global) -> Result<()> {
    if let Some(url) = &global.url {
        let path = config::save_values(&[("url", &config::normalize_url(url))])?;
        output::note(&format!("saved server url to {}", path.display()));
    }
    let client = client(global)?;
    if let crate::api::AuthStatus::Authenticated = client.try_silent_auth().await? {
        output::note("already signed in (or the server has auth disabled)");
        return Ok(());
    }
    let session = client.start_loopback().await?;
    output::note(&format!(
        "opening the browser to sign in, or open this URL:\n  {}",
        session.authorize_url
    ));
    let _ = open::that(&session.authorize_url);
    session.finish(&client).await?;
    output::note("signed in");
    Ok(())
}

async fn logout(global: &Global) -> Result<()> {
    if let Some(session) = session_store::load() {
        let client = client(global)?;
        client.set_session(Some(session));
        let _ = client.call(Method::POST, "/auth/logout", &[], None).await;
    }
    session_store::clear();
    output::note("signed out (a token in the config file or AUTO_TOKEN stays valid)");
    Ok(())
}

async fn token(ops: &Ops, command: TokenCommand, format: Format) -> Result<()> {
    match command {
        TokenCommand::Create {
            name,
            expires_in_days,
            save,
        } => {
            let body = json!({ "name": name, "expires_in_days": expires_in_days });
            let created = ops
                .client
                .call(Method::POST, "/auth/tokens", &[], Some(&body))
                .await?;
            let secret = created
                .get("secret")
                .and_then(Value::as_str)
                .ok_or_else(|| eyre!("the server did not return a token secret"))?;
            if save {
                let url = ops.client.server_url().to_string();
                let path = config::save_values(&[("url", &url), ("token", secret)])?;
                output::note(&format!("saved token to {}", path.display()));
            }
            match format {
                Format::Json => output::json(&created)?,
                _ => {
                    output::note("token created; it is shown only once:");
                    println!("{secret}");
                }
            }
        }
        TokenCommand::List => {
            let tokens = ops
                .client
                .call(Method::GET, "/auth/tokens", &[], None)
                .await?;
            match format {
                Format::Json => output::json(&tokens)?,
                _ => tokens
                    .as_array()
                    .into_iter()
                    .flatten()
                    .try_for_each(|token| {
                        output::detail(token, format)?;
                        println!();
                        Ok::<_, io::Error>(())
                    })?,
            }
        }
        TokenCommand::Revoke { id } => {
            ops.client
                .call(Method::DELETE, &format!("/auth/tokens/{id}"), &[], None)
                .await?;
            output::note("token revoked");
        }
    }
    Ok(())
}
