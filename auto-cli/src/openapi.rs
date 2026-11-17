//! Read field and relation info from the server's `/api/openapi.json`.
//!
//! The CLI and MCP never hard-code entity fields. They ask the running
//! server, so a field added in the backend shows up here without a client
//! release.

use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::api::{ApiClient, EntityKind};

/// One field of a request body or one query parameter.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    /// Server default applied when the field is omitted on create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Add allowed values and create defaults from the server's `/api/config`.
///
/// `options` keys are plural or prefixed forms of field names (`statuses`,
/// `infra_status`, `share_usages`), so a field matches the first existing key
/// of: `{entity}_{field}`, `{entity}_{field}s`, `{field}s`, `{field}es`,
/// `{field}`. `defaults.{entity}.{field}` gives the default.
pub fn apply_config(fields: &mut [Field], entity: &str, config: &Value, with_defaults: bool) {
    let options = config.get("options");
    fields.iter_mut().for_each(|field| {
        let name = &field.name;
        let values = [
            format!("{entity}_{name}"),
            format!("{entity}_{name}s"),
            format!("{name}s"),
            format!("{name}es"),
            name.clone(),
        ]
        .iter()
        .find_map(|key| options?.get(key)?.as_object());
        if field.enum_values.is_empty()
            && let Some(values) = values
        {
            field.enum_values = values.keys().cloned().collect();
        }
        if with_defaults {
            field.default = config
                .pointer(&format!("/defaults/{entity}/{name}"))
                .cloned();
        }
    });
}

/// A link relation, e.g. `applications/{id}/people/{person_id}`.
#[derive(Debug, Clone, Serialize)]
pub struct Relation {
    pub name: String,
    pub fields: Vec<Field>,
}

/// Everything the clients need to know about one entity kind.
#[derive(Debug, Clone, Serialize)]
pub struct EntitySchema {
    pub entity: String,
    pub create: Vec<Field>,
    pub update: Vec<Field>,
    pub list_filters: Vec<Field>,
    pub relations: Vec<Relation>,
}

pub struct Spec(Value);

impl Spec {
    pub async fn fetch(client: &ApiClient) -> Result<Self> {
        let doc = client
            .call(reqwest::Method::GET, "/openapi.json", &[], None)
            .await?;
        Ok(Self(doc))
    }

    pub fn entity(&self, kind: EntityKind) -> EntitySchema {
        let base = format!("/api/{}", kind.path());
        EntitySchema {
            entity: kind.path().to_string(),
            create: self.body_fields(&base, "post"),
            update: self.body_fields(&format!("{base}/{{id}}"), "put"),
            list_filters: self
                .query_params(&base, "get")
                .into_iter()
                .filter(|field| !["page", "per_page", "search"].contains(&field.name.as_str()))
                .collect(),
            relations: self.relations(kind),
        }
    }

    /// Fields of the JSON request body of `method path`.
    fn body_fields(&self, path: &str, method: &str) -> Vec<Field> {
        self.0
            .pointer(&format!(
                "/paths/{}/{method}/requestBody/content/application~1json/schema",
                escape(path)
            ))
            .map(|schema| self.object_fields(schema))
            .unwrap_or_default()
    }

    fn query_params(&self, path: &str, method: &str) -> Vec<Field> {
        self.0
            .pointer(&format!("/paths/{}/{method}/parameters", escape(path)))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|param| param.get("in").and_then(Value::as_str) == Some("query"))
            .map(|param| {
                let mut field = self.field(
                    param.get("name").and_then(Value::as_str).unwrap_or("?"),
                    param.get("schema").unwrap_or(&Value::Null),
                    param
                        .get("required")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                );
                field.description = field.description.or_else(|| text(param, "description"));
                field
            })
            .collect()
    }

    /// Link relations: every `POST /api/{entity}/{id}/{relation}/{other}`.
    fn relations(&self, kind: EntityKind) -> Vec<Relation> {
        let prefix = format!("/api/{}/{{id}}/", kind.path());
        self.0
            .get("paths")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(path, item)| {
                let rest = path.strip_prefix(&prefix)?;
                let (name, other) = rest.split_once('/')?;
                (other.starts_with('{') && !other.contains('/') && item.get("post").is_some()).then(
                    || Relation {
                        name: name.to_string(),
                        fields: self.body_fields(path, "post"),
                    },
                )
            })
            .collect()
    }

    fn resolve<'a>(&'a self, schema: &'a Value) -> &'a Value {
        schema
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|reference| reference.strip_prefix('#'))
            .and_then(|pointer| self.0.pointer(pointer))
            .unwrap_or(schema)
    }

    fn object_fields(&self, schema: &Value) -> Vec<Field> {
        let schema = self.resolve(schema);
        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        schema
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, prop)| self.field(name, prop, required.contains(&name.as_str())))
            .collect()
    }

    fn field(&self, name: &str, schema: &Value, required: bool) -> Field {
        let (ty, nullable, enum_values) = self.type_of(schema);
        Field {
            name: name.to_string(),
            ty: if nullable { format!("{ty}?") } else { ty },
            required,
            description: text(schema, "description"),
            enum_values,
            default: None,
        }
    }

    /// (type name, nullable, enum values) for a property schema, following
    /// `$ref`, `oneOf`/`anyOf` with null, and `type: [x, "null"]`.
    fn type_of(&self, schema: &Value) -> (String, bool, Vec<String>) {
        let schema = self.resolve(schema);
        if let Some(variants) = schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .and_then(Value::as_array)
        {
            let is_null = |v: &&Value| v.get("type").and_then(Value::as_str) == Some("null");
            let nullable = variants.iter().any(|v| is_null(&v));
            return match variants.iter().find(|v| !is_null(v)) {
                Some(inner) => {
                    let (ty, inner_nullable, values) = self.type_of(inner);
                    (ty, nullable || inner_nullable, values)
                }
                None => ("null".into(), true, Vec::new()),
            };
        }
        let enum_values = schema
            .get("enum")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        match schema.get("type") {
            Some(Value::String(ty)) => (array_type(self, schema, ty), false, enum_values),
            Some(Value::Array(types)) => {
                let nullable = types.iter().any(|t| t == "null");
                let ty = types
                    .iter()
                    .filter_map(Value::as_str)
                    .find(|t| *t != "null")
                    .unwrap_or("any");
                (array_type(self, schema, ty), nullable, enum_values)
            }
            _ => ("any".into(), false, enum_values),
        }
    }
}

fn array_type(spec: &Spec, schema: &Value, ty: &str) -> String {
    match (ty, schema.get("items")) {
        ("array", Some(items)) => format!("{}[]", spec.type_of(items).0),
        _ => ty.to_string(),
    }
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

/// JSON pointer escaping for a path key (`~` → `~0`, `/` → `~1`).
fn escape(path: &str) -> String {
    path.replace('~', "~0").replace('/', "~1")
}

/// Build a JSON body from `key=value` pairs, typed by the schema fields.
///
/// `key:=json` always takes the raw JSON value. Plain `key=value` is parsed by
/// the field type: numbers, booleans (`true/false/yes/no`), `null` for
/// nullable fields, comma lists for arrays. Unknown keys are an error, so a
/// typo does not get silently dropped by the server.
pub fn body_from_pairs(fields: &[Field], pairs: &[String]) -> Result<Map<String, Value>> {
    pairs
        .iter()
        .map(|pair| {
            if let Some((key, raw)) = pair.split_once(":=") {
                let value = serde_json::from_str(raw)
                    .map_err(|e| eyre!("`{key}:=` needs valid JSON: {e}"))?;
                return Ok((key.to_string(), value));
            }
            let (key, raw) = pair
                .split_once('=')
                .ok_or_else(|| eyre!("expected key=value, got `{pair}`"))?;
            let field = fields.iter().find(|f| f.name == key).ok_or_else(|| {
                let known: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                eyre!("unknown field `{key}` (known: {})", known.join(", "))
            })?;
            Ok((key.to_string(), coerce(field, raw)?))
        })
        .collect()
}

fn coerce(field: &Field, raw: &str) -> Result<Value> {
    let nullable = field.ty.ends_with('?');
    let ty = field.ty.trim_end_matches('?');
    if nullable && raw == "null" {
        return Ok(Value::Null);
    }
    let bad = || eyre!("field `{}` expects {ty}, got `{raw}`", field.name);
    Ok(match ty {
        "integer" => Value::from(raw.parse::<i64>().map_err(|_| bad())?),
        "number" => Value::from(raw.parse::<f64>().map_err(|_| bad())?),
        "boolean" => Value::Bool(match raw.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => true,
            "false" | "no" | "0" | "off" => false,
            _ => return Err(bad()),
        }),
        ty if ty.ends_with("[]") => Value::Array(
            raw.split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(|item| Value::String(item.to_string()))
                .collect(),
        ),
        "object" | "any" => serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.into())),
        _ => Value::String(raw.to_string()),
    })
}
