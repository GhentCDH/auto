use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;
use utoipa::ToSchema;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub database_url: String,
    pub kuma_url: Url,
    pub kuma_username: String,
    pub kuma_password: String,
    #[serde(default)]
    pub kuma_notification_name: Option<String>,
    #[serde(default)]
    pub outline_url: Option<Url>,
    #[serde(default)]
    pub outline_api_key: Option<String>,
    /// How many days before an infra's domain-resolved IPs are considered stale
    /// and re-resolved on read. Optional `INFRA_IP_REFRESH_DAYS` (default 10).
    #[serde(default = "default_infra_ip_refresh_days")]
    pub infra_ip_refresh_days: u64,
    /// Configurable defaults applied to entity creation and surfaced to the
    /// frontend for form pre-fill. Configured under `[defaults.*]` in `auto.toml`.
    #[serde(default)]
    pub defaults: Defaults,
    /// Configurable dropdown option lists (value -> label), surfaced to the
    /// frontend. Configured under `[options.*]` in `auto.toml`.
    #[serde(default)]
    pub options: Options,
}

fn default_infra_ip_refresh_days() -> u64 {
    10
}

/// Non-secret configuration served to the frontend at `GET /api/config`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicConfig {
    pub defaults: Defaults,
    pub options: Options,
}

impl From<&Config> for PublicConfig {
    fn from(c: &Config) -> Self {
        Self {
            defaults: c.defaults.clone(),
            options: c.options.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Defaults — single source of truth for entity-creation defaults. Backend
// create handlers fill omitted fields from these; the frontend reads the same
// values to pre-fill forms. Built-in values reproduce prior behavior, so a
// missing `auto.toml` changes nothing.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct Defaults {
    #[serde(default)]
    pub application: ApplicationDefaults,
    #[serde(default)]
    pub service: ServiceDefaults,
    #[serde(default)]
    pub healthcheck: HealthcheckDefaults,
    #[serde(default)]
    pub note: NoteDefaults,
    #[serde(default)]
    pub person: PersonDefaults,
    #[serde(default)]
    pub share: ShareDefaults,
    #[serde(default)]
    pub infra: InfraDefaults,
    #[serde(default)]
    pub domain: DomainDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApplicationDefaults {
    pub environment: String,
    pub status: String,
}

impl Default for ApplicationDefaults {
    fn default() -> Self {
        Self {
            environment: "prd".into(),
            status: "active".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceDefaults {
    pub environment: String,
    pub status: String,
}

impl Default for ServiceDefaults {
    fn default() -> Self {
        Self {
            environment: "prd".into(),
            status: "active".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthcheckDefaults {
    pub protocol: String,
    pub path: String,
    pub method: String,
    pub expected_status: i32,
    pub timeout_seconds: i32,
    pub interval: i32,
    pub is_enabled: bool,
    pub notifications: bool,
    pub retry: i32,
    pub retry_interval: i32,
    pub request_body_encoding: String,
}

impl Default for HealthcheckDefaults {
    fn default() -> Self {
        Self {
            protocol: "https".into(),
            path: "/".into(),
            method: "GET".into(),
            expected_status: 200,
            timeout_seconds: 30,
            interval: 60,
            is_enabled: true,
            notifications: true,
            retry: 0,
            retry_interval: 60,
            request_body_encoding: "JSON".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteDefaults {
    pub note_type: String,
}

impl Default for NoteDefaults {
    fn default() -> Self {
        Self {
            note_type: "general".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonDefaults {
    pub department: String,
    pub email_domain: String,
    pub contribution_type: String,
    pub is_active: bool,
}

impl Default for PersonDefaults {
    fn default() -> Self {
        Self {
            department: "GhentCDH".into(),
            email_domain: "ugent.be".into(),
            contribution_type: "developer".into(),
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShareDefaults {
    pub path_prefix: String,
    pub server: String,
    pub share_type: String,
    pub status: String,
    pub usage: String,
    pub permissions: String,
}

impl Default for ShareDefaults {
    fn default() -> Self {
        Self {
            path_prefix: "/ghentcdh_".into(),
            server: "files.ugent.be".into(),
            share_type: "smb".into(),
            status: "active".into(),
            usage: "data".into(),
            permissions: "read-write".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InfraDefaults {
    /// Name prefix that auto-selects the "vm" infra type in the create form.
    pub vm_name_prefix: String,
}

impl Default for InfraDefaults {
    fn default() -> Self {
        Self {
            vm_name_prefix: "gcdh".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DomainDefaults {
    pub target_type: String,
}

impl Default for DomainDefaults {
    fn default() -> Self {
        Self {
            target_type: "application".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Options — dropdown value -> label maps. `IndexMap` preserves the order in
// which entries are declared (TOML order), so dropdowns stay stable.
// ---------------------------------------------------------------------------

/// Ordered value -> label map for a dropdown.
type OptionList = IndexMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Options {
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub statuses: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub environments: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub infra_types: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub share_usages: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub share_types: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub domain_types: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub domain_status: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub contribution_types: OptionList,
    #[schema(value_type = std::collections::HashMap<String, String>)]
    pub note_types: OptionList,
}

fn map(pairs: &[(&str, &str)]) -> OptionList {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

impl Default for Options {
    fn default() -> Self {
        Self {
            statuses: map(&[
                ("active", "Active"),
                ("inactive", "Inactive"),
                ("deprecated", "Deprecated"),
                ("archived", "Archived"),
            ]),
            environments: map(&[
                ("prd", "Production"),
                ("dev", "Development"),
                ("qas", "Quality Assurance"),
                ("tst", "Testing"),
            ]),
            infra_types: map(&[
                ("nomad_cluster", "Nomad Cluster"),
                ("server", "Server"),
                ("vm", "Virtual Machine"),
            ]),
            share_usages: map(&[
                ("data", "Data Storage"),
                ("config", "Configuration"),
                ("logs", "Logs"),
                ("backup", "Backup"),
                ("media", "Media"),
            ]),
            share_types: map(&[("smb", "SMB"), ("nfs", "NFS")]),
            domain_types: map(&[
                ("A", "A"),
                ("AAAA", "AAAA"),
                ("CNAME", "CNAME"),
                ("MX", "MX"),
                ("TXT", "TXT"),
            ]),
            domain_status: map(&[
                ("active", "Active"),
                ("inactive", "Inactive"),
                ("expired", "Expired"),
            ]),
            contribution_types: map(&[
                ("project_owner", "Project Owner"),
                ("developer", "Developer"),
                ("maintainer", "Maintainer"),
                ("stakeholder", "Stakeholder"),
            ]),
            note_types: map(&[
                ("general", "General"),
                ("documentation", "Documentation"),
                ("changelog", "Changelog"),
            ]),
        }
    }
}

impl Config {
    /// Load configuration by layering an optional `auto.toml` file under the
    /// process environment. Environment variables override TOML on conflict.
    ///
    /// `.env` (falling back to `dev.env`) is loaded into the process environment
    /// first, so file-based secrets keep working.
    ///
    /// # Errors
    /// If a required setting is missing or a value has the wrong format.
    pub fn load() -> Result<Self, Error> {
        info!("loading configuration (auto.toml < env)");
        if dotenvy::dotenv().is_err() {
            info!(".env not found, defaulting to dev.env");
            if dotenvy::from_path("dev.env").is_err() {
                info!("dev.env not found");
            }
        }

        Figment::new()
            .merge(Toml::file("auto.toml"))
            .merge(Env::raw())
            .extract()
            .map_err(Error::from)
    }
}
