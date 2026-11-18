//! Generic operations on the auto API, shared by the CLI and the MCP server.
//!
//! Every entity uses the same REST shape (`/api/{entity}`, `/{id}`,
//! `/{id}/{relation}/{other}`), so one set of functions covers all of them.

use color_eyre::eyre::{Result, WrapErr, eyre};
use futures::FutureExt;
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::OnceCell;

use crate::api::{ApiClient, ApiError, EntityKind};
use crate::openapi::{self, EntitySchema, Spec};
use crate::view::row_label;

/// A write request, built first so it can be shown (dry run) or sent.
#[derive(Debug, Clone, Serialize)]
pub struct Request {
    #[serde(serialize_with = "serialize_method")]
    pub method: Method,
    /// Path under `/api`, shown with the prefix.
    #[serde(serialize_with = "serialize_path")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

fn serialize_method<S: serde::Serializer>(method: &Method, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(method.as_str())
}

fn serialize_path<S: serde::Serializer>(path: &str, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&format_args!("/api{path}"))
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} /api{}", self.method, self.path)?;
        match &self.body {
            Some(body) => write!(f, " {body}"),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ListQuery {
    pub search: Option<String>,
    /// Extra query filters such as `status` or `environment`.
    pub filters: Vec<(String, String)>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    /// Fetch every page and return them as one.
    pub all: bool,
}

/// Server-side actions that are not plain CRUD.
#[derive(Debug, Clone)]
pub enum Action {
    Dns { domain: String },
    RunHealthcheck { id: String },
    SyncInfra { id: Option<String> },
    SyncKuma { id: Option<String> },
    SyncDomains,
    SyncOutline { kind: EntityKind, id: String },
}

pub struct Ops {
    pub client: ApiClient,
    spec: OnceCell<Spec>,
    config: OnceCell<Value>,
}

impl Ops {
    pub fn new(client: ApiClient) -> Self {
        Self {
            client,
            spec: OnceCell::new(),
            config: OnceCell::new(),
        }
    }

    /// The OpenAPI spec, fetched once per process on first use.
    pub async fn spec(&self) -> Result<&Spec> {
        self.spec
            .get_or_try_init(|| Spec::fetch(&self.client))
            .await
    }

    async fn get_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        self.client.call(Method::GET, path, query, None).await
    }

    pub async fn list(&self, kind: EntityKind, query: &ListQuery) -> Result<Value> {
        if !query.all {
            return self.list_page(kind, query, query.page.unwrap_or(1)).await;
        }
        // The first page tells how many there are; fetch the rest together.
        let first = self.list_page(kind, query, 1).await?;
        let total_pages = first
            .get("total_pages")
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32;
        let rest = futures::future::try_join_all(
            (2..=total_pages).map(|page| self.list_page(kind, query, page)),
        )
        .await?;
        let rows: Vec<Value> = std::iter::once(first)
            .chain(rest)
            .flat_map(
                |mut response| match response.get_mut("data").map(Value::take) {
                    Some(Value::Array(rows)) => rows,
                    _ => Vec::new(),
                },
            )
            .collect();
        let total = rows.len();
        Ok(json!({
            "data": rows,
            "total": total,
            "page": 1,
            "per_page": total,
            "total_pages": 1,
        }))
    }

    async fn list_page(&self, kind: EntityKind, query: &ListQuery, page: u32) -> Result<Value> {
        let per_page = query.per_page.unwrap_or(if query.all { 100 } else { 50 });
        let params: Vec<(String, String)> = [
            ("page".to_string(), page.to_string()),
            ("per_page".to_string(), per_page.to_string()),
        ]
        .into_iter()
        .chain(
            query
                .search
                .iter()
                .map(|s| ("search".to_string(), s.clone())),
        )
        .chain(query.filters.iter().cloned())
        .collect();
        self.get_json(&format!("/{}", kind.path()), &params).await
    }

    /// Turn an id, id prefix or exact name into a full id.
    ///
    /// Tries, in order: the key as a full id; the key as an id prefix via
    /// `/api/resolve/{key}` (only some entity handlers expand prefixes
    /// themselves); a search for a row whose name (or fqdn/title) matches
    /// exactly, ignoring case.
    pub async fn resolve(&self, kind: EntityKind, key: &str) -> Result<(String, Value)> {
        match self.get_json(&format!("/{}/{key}", kind.path()), &[]).await {
            Ok(value) => {
                let id = value
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or(key)
                    .to_string();
                return Ok((id, value));
            }
            Err(error) if is_status(&error, 404) => {}
            Err(error) => return Err(error),
        }

        if let Some(id) = self.resolve_prefix(kind, key).await? {
            let value = self
                .get_json(&format!("/{}/{id}", kind.path()), &[])
                .await?;
            return Ok((id, value));
        }

        let query = ListQuery {
            search: Some(key.to_string()),
            per_page: Some(100),
            ..Default::default()
        };
        let response = self.list(kind, &query).await?;
        let rows = response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let matches: Vec<&Value> = rows
            .iter()
            .filter(|row| row_label(row).eq_ignore_ascii_case(key))
            .collect();
        match matches.as_slice() {
            [row] => {
                let id = row
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| eyre!("{kind} `{key}` has no id"))?
                    .to_string();
                let value = self
                    .get_json(&format!("/{}/{id}", kind.path()), &[])
                    .await?;
                Ok((id, value))
            }
            [] => {
                let close: Vec<&str> = rows.iter().take(5).map(row_label).collect();
                Err(match close.is_empty() {
                    true => eyre!("no {kind} matches `{key}`"),
                    false => eyre!(
                        "no {kind} named `{key}`; close matches: {}",
                        close.join(", ")
                    ),
                })
            }
            many => {
                let ids: Vec<String> = many
                    .iter()
                    .map(|row| {
                        let id = row.get("id").and_then(Value::as_str).unwrap_or("?");
                        let env = row.get("environment").and_then(Value::as_str).unwrap_or("");
                        format!("{id} {env}")
                    })
                    .collect();
                Err(eyre!(
                    "`{key}` matches {} {kind}; use an id: {}",
                    many.len(),
                    ids.join(", ")
                ))
            }
        }
    }

    /// Expand an id prefix through `/api/resolve/{key}`. `None` when the key
    /// does not look like an id or nothing matches, so a name lookup can
    /// follow. An id of another entity kind is an error that names the kind.
    async fn resolve_prefix(&self, kind: EntityKind, key: &str) -> Result<Option<String>> {
        let looks_like_id =
            key.len() >= 4 && key.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
        if !looks_like_id {
            return Ok(None);
        }
        let resolved = match self.get_json(&format!("/resolve/{key}"), &[]).await {
            Ok(resolved) => resolved,
            Err(error) if is_status(&error, 404) => return Ok(None),
            Err(error) => return Err(error),
        };
        let text = |field| {
            resolved
                .get(field)
                .and_then(Value::as_str)
                .unwrap_or_default()
        };
        // The resolve endpoint names shares `network_share`.
        let expected = match kind {
            EntityKind::Shares => "network_share",
            kind => kind.singular(),
        };
        match text("entity_type") {
            found if found == expected => Ok(Some(text("id").to_string())),
            found => Err(eyre!(
                "`{key}` is a {found} (`{}`), not one of {kind}",
                text("name")
            )),
        }
    }

    pub async fn get(&self, kind: EntityKind, key: &str) -> Result<Value> {
        Ok(self.resolve(kind, key).await?.1)
    }

    pub async fn id(&self, kind: EntityKind, key: &str) -> Result<String> {
        Ok(self.resolve(kind, key).await?.0)
    }

    /// Fields that point at another entity (`service_id`, `target_infra_id`,
    /// `parent_id`, a note's `entity_id`) may hold an exact name or an id
    /// prefix, so the person who approves the request can read it. Replace
    /// those with full ids.
    async fn resolve_refs(&self, kind: EntityKind, mut body: Value) -> Result<Value> {
        let Some(fields) = body.as_object() else {
            return Ok(body);
        };
        let refs: Vec<(String, EntityKind, String)> = fields
            .iter()
            .filter_map(|(field, value)| {
                let key = value.as_str().filter(|key| !is_full_id(key))?;
                Some((
                    field.clone(),
                    ref_kind(kind, field, fields)?,
                    key.to_string(),
                ))
            })
            .collect();
        let ids = futures::future::try_join_all(refs.iter().map(|(field, target, key)| {
            self.id(*target, key)
                .map(move |id| id.wrap_err_with(|| format!("`{field}`")))
        }))
        .await?;
        if let Some(fields) = body.as_object_mut() {
            refs.into_iter().zip(ids).for_each(|((field, ..), id)| {
                fields.insert(field, Value::String(id));
            });
        }
        Ok(body)
    }

    /// Current state of every healthcheck (from Uptime Kuma), worst first.
    /// `states` filters, e.g. `["down", "pending"]`.
    pub async fn status(&self, states: &[String]) -> Result<Value> {
        let query: Vec<(String, String)> = match states.is_empty() {
            true => Vec::new(),
            false => vec![("state".to_string(), states.join(","))],
        };
        self.get_json("/healthchecks/status", &query)
            .await
            .map_err(|error| match is_status(&error, 404) {
                true => eyre!("the server is too old for `status`: it needs auto 1.16.0 or newer"),
                false => error,
            })
    }

    pub async fn search(&self, query: &str) -> Result<Value> {
        self.get_json("/search", &[("q".to_string(), query.to_string())])
            .await
    }

    /// Fields from the OpenAPI spec, with allowed values and defaults from
    /// `/api/config`.
    pub async fn schema(&self, kind: EntityKind) -> Result<EntitySchema> {
        let (spec, config) = tokio::try_join!(self.spec(), self.public_config())?;
        let mut schema = spec.entity(kind);
        let entity = kind.singular();
        openapi::apply_config(&mut schema.create, entity, config, true);
        openapi::apply_config(&mut schema.update, entity, config, false);
        openapi::apply_config(&mut schema.list_filters, entity, config, false);
        schema.relations.iter_mut().for_each(|relation| {
            let target = EntityKind::from_name(&relation.name).map_or(entity, EntityKind::singular);
            openapi::apply_config(&mut relation.fields, target, config, true);
        });
        Ok(schema)
    }

    async fn public_config(&self) -> Result<&Value> {
        self.config
            .get_or_try_init(|| self.get_json("/config", &[]))
            .await
    }

    pub async fn overview(&self, kind: EntityKind, key: &str) -> Result<String> {
        if !matches!(kind, EntityKind::Applications | EntityKind::Services) {
            return Err(eyre!("overview exists only for applications and services"));
        }
        let id = self.id(kind, key).await?;
        let body = self
            .get_json(&format!("/{}/{id}/overview.md", kind.path()), &[])
            .await?;
        Ok(match body {
            Value::String(text) => text,
            other => other.to_string(),
        })
    }

    pub async fn create(&self, kind: EntityKind, body: Value) -> Result<Request> {
        Ok(Request {
            method: Method::POST,
            path: format!("/{}", kind.path()),
            body: Some(self.resolve_refs(kind, body).await?),
        })
    }

    pub async fn update(&self, kind: EntityKind, key: &str, body: Value) -> Result<Request> {
        let (id, body) = tokio::try_join!(self.id(kind, key), self.resolve_refs(kind, body))?;
        Ok(Request {
            method: Method::PUT,
            path: format!("/{}/{id}", kind.path()),
            body: Some(body),
        })
    }

    pub async fn delete(&self, kind: EntityKind, key: &str) -> Result<Request> {
        let id = self.id(kind, key).await?;
        Ok(Request {
            method: Method::DELETE,
            path: format!("/{}/{id}", kind.path()),
            body: None,
        })
    }

    /// Link (`unlink = false`) or unlink two entities. The relation is the
    /// target's entity kind, as in `/applications/{id}/people/{person_id}`.
    pub async fn link(
        &self,
        kind: EntityKind,
        key: &str,
        relation: EntityKind,
        other: &str,
        body: Option<Value>,
        unlink: bool,
    ) -> Result<Request> {
        let spec = self.spec().await?;
        let relations = spec.entity(kind).relations;
        if !relations.iter().any(|r| r.name == relation.path()) {
            let reverse = spec
                .entity(relation)
                .relations
                .iter()
                .any(|r| r.name == kind.path());
            let known: Vec<&str> = relations.iter().map(|r| r.name.as_str()).collect();
            return Err(match (reverse, known.is_empty()) {
                (true, _) => eyre!(
                    "links are made from the other side: use `link {relation} <{}> {kind} <{}>`",
                    relation.singular(),
                    kind.singular()
                ),
                (false, true) => eyre!("{kind} has no link relations"),
                (false, false) => {
                    eyre!(
                        "{kind} cannot link {relation} (can link: {})",
                        known.join(", ")
                    )
                }
            });
        }
        let (id, other_id) = tokio::try_join!(self.id(kind, key), self.id(relation, other))?;
        Ok(Request {
            method: if unlink { Method::DELETE } else { Method::POST },
            path: format!("/{}/{id}/{}/{other_id}", kind.path(), relation.path()),
            body: if unlink {
                None
            } else {
                Some(body.unwrap_or_else(|| json!({})))
            },
        })
    }

    pub async fn action(&self, action: Action) -> Result<Value> {
        let (method, path) = match action {
            Action::Dns { domain } => {
                let id = self.id(EntityKind::Domains, &domain).await?;
                (Method::GET, format!("/domains/{id}/dns"))
            }
            Action::RunHealthcheck { id } => {
                let id = self.id(EntityKind::Healthchecks, &id).await?;
                (Method::GET, format!("/healthchecks/{id}/execute"))
            }
            Action::SyncInfra { id: None } => (Method::POST, "/infra/sync".into()),
            Action::SyncInfra { id: Some(key) } => {
                let id = self.id(EntityKind::Infra, &key).await?;
                (Method::POST, format!("/infra/{id}/sync"))
            }
            Action::SyncKuma { id: None } => (Method::POST, "/healthchecks/sync/kuma".into()),
            Action::SyncKuma { id: Some(key) } => {
                let id = self.id(EntityKind::Healthchecks, &key).await?;
                (Method::POST, format!("/healthchecks/sync/kuma/{id}"))
            }
            Action::SyncDomains => (Method::POST, "/domains/sync".into()),
            Action::SyncOutline { kind, id } => {
                if !matches!(kind, EntityKind::Applications | EntityKind::Services) {
                    return Err(eyre!(
                        "outline sync exists only for applications and services"
                    ));
                }
                let id = self.id(kind, &id).await?;
                (Method::POST, format!("/{}/{id}/sync-outline", kind.path()))
            }
        };
        self.client.call(method, &path, &[], None).await
    }

    /// Send a write request, or with `dry_run` return what would be sent.
    pub async fn send(&self, request: &Request, dry_run: bool) -> Result<Value> {
        if dry_run {
            return Ok(json!({ "dry_run": true, "request": request }));
        }
        let result = self
            .client
            .call(
                request.method.clone(),
                &request.path,
                &[],
                request.body.as_ref(),
            )
            .await?;
        // 204 responses (delete, link) carry no body: report what was done.
        Ok(match result {
            Value::Null => json!({ "ok": true, "request": request }),
            result => result,
        })
    }
}

/// A complete UUID, which needs no lookup.
fn is_full_id(key: &str) -> bool {
    key.len() == 36
        && key.char_indices().all(|(i, c)| {
            matches!(i, 8 | 13 | 18 | 23) == (c == '-') && (c == '-' || c.is_ascii_hexdigit())
        })
}

/// The entity kind a reference field points at: `target_service_id` and
/// `service_id` are services, `parent_id` is the same kind, and a note's
/// `entity_id` follows its `entity_type`.
fn ref_kind(
    kind: EntityKind,
    field: &str,
    body: &serde_json::Map<String, Value>,
) -> Option<EntityKind> {
    match field.strip_suffix("_id")? {
        "parent" => Some(kind),
        "entity" => EntityKind::from_name(body.get("entity_type")?.as_str()?),
        name => EntityKind::from_name(name.strip_prefix("target_").unwrap_or(name)),
    }
}

pub fn is_status(error: &color_eyre::Report, status: u16) -> bool {
    error
        .downcast_ref::<ApiError>()
        .is_some_and(|e| e.status == status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_ids() {
        assert!(is_full_id("65e6cf93-cd47-4654-9480-f907e9fe7c5e"));
        assert!(!is_full_id("65e6cf93"));
        assert!(!is_full_id("Caddy LB"));
        assert!(!is_full_id("65e6cf93-cd47-4654-9480-f907e9fe7c5"));
        assert!(!is_full_id("65e6cf93cd47-4654-9480-f907e9fe7c5e-"));
    }

    #[test]
    fn reference_fields() {
        let note = json!({ "entity_type": "network_share", "entity_id": "x", "title": "t" });
        let note = note.as_object().unwrap();
        let kind = |field| ref_kind(EntityKind::Infra, field, note);
        assert_eq!(kind("entity_id"), Some(EntityKind::Shares));
        assert_eq!(kind("target_service_id"), Some(EntityKind::Services));
        assert_eq!(kind("application_id"), Some(EntityKind::Applications));
        assert_eq!(kind("domain_id"), Some(EntityKind::Domains));
        assert_eq!(kind("parent_id"), Some(EntityKind::Infra));
        assert_eq!(kind("kuma_id"), None);
        assert_eq!(kind("title"), None);
    }
}
