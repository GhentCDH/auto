//! Which auto application or service is developed in the current git repo?
//!
//! Used by `auto here`, by the audit skill, and by the plugin hooks that make
//! Claude suggest auto updates when deployment-relevant files change.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use color_eyre::eyre::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::api::{ApiClient, EntityKind};
use crate::config::{APP_DIR, Config, DEPLOY_REPOS_HELP};
use crate::ops::{ListQuery, Ops, is_status};
use crate::view::row_label;

/// How long a hook may reuse a repo lookup before asking the server again.
const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
/// Hooks must never slow a session down noticeably.
const HOOK_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub root: PathBuf,
    pub remote: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub entity: String,
    pub id: String,
    pub name: String,
    pub environment: Option<String>,
    /// `repository_url` (certain) or `name` (a guess to confirm).
    pub matched_by: String,
}

/// A local checkout of a central deploy repo (`deploy_repos` in the config).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployRepo {
    pub path: PathBuf,
    pub exists: bool,
    /// What it holds, from its file names: `caddy`, `nomad`, `compose`.
    #[serde(default)]
    pub kinds: Vec<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    /// Date of the checked-out commit.
    pub last_commit: Option<String>,
    /// Uncommitted local changes.
    pub dirty: Option<bool>,
    /// Upstream commits that are not checked out, as of the last fetch.
    pub behind: Option<u32>,
    /// Whether `--fetch` refreshed it just now (`None`: not attempted).
    pub fetched: Option<bool>,
}

/// The signed-in user's role on the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

/// The user's role; `None` when it cannot be read. A server without auth
/// has no `/auth/me` and lets everyone do everything, like an admin.
pub async fn user_role(client: &ApiClient) -> Option<Role> {
    match client.call(Method::GET, "/auth/me", &[], None).await {
        Ok(me) => Role::deserialize(me.get("role")?).ok(),
        Err(error) if is_status(&error, 404) => Some(Role::Admin),
        Err(_) => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Here {
    pub repo: Repo,
    pub matches: Vec<Match>,
    pub deploy_repos: Vec<DeployRepo>,
}

/// Output of a git command in `dir`, trimmed; `None` on failure or no output.
fn git_in(dir: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// Describe the deploy repos; with `fetch`, run `git fetch` in each first
/// (at most 15 seconds each, all at once) so `behind` is current.
pub async fn deploy_repos(paths: &[PathBuf], fetch: bool) -> Vec<DeployRepo> {
    let fetched: Vec<Option<bool>> = match fetch {
        true => futures::future::join_all(paths.iter().map(|path| fetch_repo(path))).await,
        false => vec![None; paths.len()],
    };
    paths
        .iter()
        .zip(fetched)
        .map(|(path, fetched)| DeployRepo {
            fetched,
            ..deploy_repo(path)
        })
        .collect()
}

async fn fetch_repo(path: &Path) -> Option<bool> {
    if !path.is_dir() {
        return None;
    }
    let status = tokio::process::Command::new("git")
        .args(["fetch", "--quiet"])
        .current_dir(path)
        // Never wait for a password prompt; fail instead.
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .status();
    let done = tokio::time::timeout(Duration::from_secs(15), status).await;
    Some(matches!(done, Ok(Ok(status)) if status.success()))
}

fn deploy_repo(path: &Path) -> DeployRepo {
    if !path.is_dir() {
        return DeployRepo {
            path: path.to_path_buf(),
            ..Default::default()
        };
    }
    let git = |args: &[&str]| git_in(path, args);
    let files = git(&["ls-files"]).unwrap_or_default().to_lowercase();
    // File name without a template suffix: `job.nomad.hcl.j2` -> `job.nomad.hcl`.
    let has = |test: &dyn Fn(&str) -> bool| {
        files.lines().any(|file| {
            let name = file.rsplit('/').next().unwrap_or(file);
            let name = [".j2", ".jinja", ".tpl", ".tmpl"]
                .iter()
                .find_map(|suffix| name.strip_suffix(suffix))
                .unwrap_or(name);
            test(name)
        })
    };
    let kinds = [
        (
            "caddy",
            has(&|f| f.starts_with("caddyfile") || f.ends_with(".caddy")),
        ),
        (
            "nomad",
            has(&|f| f.ends_with(".nomad") || f.ends_with(".nomad.hcl")),
        ),
        (
            "compose",
            has(&|f| f.contains("docker-compose") || f.starts_with("compose.")),
        ),
    ]
    .into_iter()
    .filter(|(_, found)| *found)
    .map(|(kind, _)| kind.to_string())
    .collect();
    DeployRepo {
        path: path.to_path_buf(),
        exists: true,
        kinds,
        remote: git(&["remote", "get-url", "origin"]),
        branch: git(&["rev-parse", "--abbrev-ref", "HEAD"]),
        last_commit: git(&["log", "-1", "--format=%cs"]),
        dirty: Some(git(&["status", "--porcelain"]).is_some()),
        behind: git(&["rev-list", "--count", "HEAD..@{upstream}"]).and_then(|n| n.parse().ok()),
        fetched: None,
    }
}

/// The git repository that contains `dir`, if any.
pub fn repo_at(dir: &Path) -> Option<Repo> {
    let git = |args: &[&str]| git_in(dir, args);
    let root = PathBuf::from(git(&["rev-parse", "--show-toplevel"])?);
    let remote = git(&["remote", "get-url", "origin"]).or_else(|| {
        git(&["remote"])
            .and_then(|remotes| remotes.lines().next().map(str::to_string))
            .and_then(|first| git(&["remote", "get-url", &first]))
    });
    let name = remote
        .as_deref()
        .and_then(|url| url.trim_end_matches('/').rsplit(['/', ':']).next())
        .map(|last| last.trim_end_matches(".git").to_string())
        .or_else(|| root.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default();
    Some(Repo { root, remote, name })
}

/// `host/owner/repo` in lower case, for comparing ssh and https remotes.
pub fn normalize_repo_url(url: &str) -> Option<String> {
    let url = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest.to_string(),
        // scp-like `git@host:owner/repo`
        None => url.replacen(':', "/", 1),
    };
    let rest = rest
        .rsplit_once('@')
        .map_or(rest.as_str(), |(_, host)| host);
    let rest = rest
        .split_once('/')
        .map_or(rest.to_string(), |(host, path)| {
            // drop a port (`host:2222`)
            format!("{}/{path}", host.split(':').next().unwrap_or(host))
        });
    (rest.matches('/').count() >= 2).then(|| rest.to_lowercase())
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn names_match(a: &str, b: &str) -> bool {
    let (a, b) = (normalize_name(a), normalize_name(b));
    !a.is_empty()
        && !b.is_empty()
        && (a == b || (a.len().min(b.len()) >= 4 && (a.contains(&b) || b.contains(&a))))
}

/// Match a repo against all applications and services.
pub async fn find(ops: &Ops, repo: &Repo, deploy_repos: &[PathBuf]) -> Result<Here> {
    let query = ListQuery {
        all: true,
        ..Default::default()
    };
    let (apps, services) = tokio::try_join!(
        ops.list(EntityKind::Applications, &query),
        ops.list(EntityKind::Services, &query)
    )?;
    let rows: Vec<(EntityKind, &Value)> = [
        (EntityKind::Applications, &apps),
        (EntityKind::Services, &services),
    ]
    .into_iter()
    .flat_map(|(kind, response)| {
        response
            .get("data")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(move |row| (kind, row))
    })
    .collect();

    let to_match = |(kind, row): &(EntityKind, &Value), by: &str| Match {
        entity: kind.path().to_string(),
        id: row
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        name: row_label(row).to_string(),
        environment: row
            .get("environment")
            .and_then(Value::as_str)
            .map(str::to_string),
        matched_by: by.to_string(),
    };
    let remote = repo.remote.as_deref().and_then(normalize_repo_url);
    let by_url: Vec<Match> = rows
        .iter()
        .filter(|(_, row)| {
            let url = row.get("repository_url").and_then(Value::as_str);
            remote.is_some() && url.and_then(normalize_repo_url) == remote
        })
        .map(|entry| to_match(entry, "repository_url"))
        .collect();
    let matches = match by_url.is_empty() {
        false => by_url,
        true => rows
            .iter()
            .filter(|(_, row)| names_match(row_label(row), &repo.name))
            .map(|entry| to_match(entry, "name"))
            .collect(),
    };

    Ok(Here {
        repo: repo.clone(),
        matches,
        deploy_repos: deploy_repos.iter().map(|path| deploy_repo(path)).collect(),
    })
}

// ---- hooks -----------------------------------------------------------------

/// Input Claude Code passes to a hook on stdin (only the fields we use).
#[derive(Debug, Default, Deserialize)]
struct HookInput {
    #[serde(default)]
    session_id: String,
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_input: Value,
}

/// Run a hook. Hooks never fail and never print errors: when anything is
/// missing (no git repo, no server, no match), they stay silent.
pub async fn run_hook(event: &str, url: Option<&str>) {
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    let input: HookInput = serde_json::from_str(&stdin).unwrap_or_default();
    let context = tokio::time::timeout(HOOK_TIMEOUT, hook_context(event, &input, url))
        .await
        .ok()
        .flatten();
    if let Some(context) = context {
        let name = match event {
            "session-start" => "SessionStart",
            _ => "PostToolUse",
        };
        let output = json!({
            "hookSpecificOutput": { "hookEventName": name, "additionalContext": context }
        });
        println!("{output}");
    }
}

async fn hook_context(event: &str, input: &HookInput, url: Option<&str>) -> Option<String> {
    let cwd = input.cwd.clone().or_else(|| std::env::current_dir().ok())?;
    let repo = repo_at(&cwd)?;
    match event {
        "session-start" => {
            let (here, role) = cached_here(&repo, url).await?;
            let first = here.matches.first()?;
            let context = session_context(&here, first, role);
            // `deploy_repos = []` is an explicit "I have none": only an absent key asks.
            let ask_deploy_repos =
                role == Role::Admin && Config::deploy_repo_setting().is_ok_and(|s| s.is_none());
            Some(match ask_deploy_repos {
                true => format!(
                    "{context}\n\nThe user has no deploy repos configured. {DEPLOY_REPOS_HELP} \
                     Tell the user this once in this session, in two or three sentences, \
                     and offer to add the paths of their checkouts."
                ),
                false => context,
            })
        }
        "post-edit" => {
            let file = input.tool_input.get("file_path").and_then(Value::as_str)?;
            if !is_deploy_file(file) {
                return None;
            }
            let (here, role) = cached_here(&repo, url).await?;
            let first = here.matches.first()?;
            if role != Role::Admin || !remind_once(&input.session_id, &repo) {
                return None;
            }
            Some(format!(
                "You changed {file}, which can change how {} is deployed or reached. When \
                 this change is done, tell the user which part of its auto record may need \
                 an update (servers, domains, services, healthchecks) and offer the \
                 auto:audit skill. Do not change auto without the user's confirmation.",
                first.name
            ))
        }
        _ => None,
    }
}

fn session_context(here: &Here, first: &Match, role: Role) -> String {
    let records: Vec<String> = here
        .matches
        .iter()
        .map(|m| {
            let env = m
                .environment
                .as_deref()
                .map(|e| format!(", {e}"))
                .unwrap_or_default();
            format!("{} `{}`{env} (id {})", m.entity, m.name, m.id)
        })
        .collect();
    let certainty = match first.matched_by.as_str() {
        "repository_url" => "",
        _ => " (matched by name only: confirm with the user before relying on it)",
    };
    let records = records.join(", ");
    let intro = format!(
        "This repository is in auto, the organization's IT inventory, as {records}{certainty}."
    );
    match role {
        Role::Viewer => format!(
            "{intro} The user can only read auto (viewer role): use it to answer questions \
             about where this runs, its domains, uptime and people. Do not propose changes \
             to auto or the auto:audit skill."
        ),
        Role::Admin | Role::Editor => format!(
            "{intro} Keep auto in sync: when work in this session changes how it is deployed \
             or reached (domains, servers or clusters, services it depends on such as \
             databases, health endpoints, ports), tell the user that auto should be updated \
             and offer the auto:audit skill, which compares the repo with auto. Do not change \
             auto without the user's confirmation."
        ),
    }
}

/// Files whose changes usually mean the deployment changed.
fn is_deploy_file(path: &str) -> bool {
    let path = path.to_lowercase();
    let file = path.rsplit('/').next().unwrap_or(&path);
    let by_name = file.ends_with(".nomad")
        || file.ends_with(".nomad.hcl")
        || file.ends_with(".tf")
        || file.contains("docker-compose")
        || (file.starts_with("compose.") && (file.ends_with(".yml") || file.ends_with(".yaml")))
        || file.starts_with("caddyfile")
        || file.starts_with("dockerfile")
        || file.ends_with(".env.example")
        || [
            "fly.toml",
            "netlify.toml",
            "vercel.json",
            "procfile",
            "app.yaml",
        ]
        .contains(&file);
    let by_dir = [
        "/.github/workflows/",
        "/deploy/",
        "/k8s/",
        "/helm/",
        "/nomad/",
        "/terraform/",
    ]
    .iter()
    .any(|dir| path.contains(dir));
    by_name || by_dir
}

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|dir| dir.join(APP_DIR))
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Remind only once per session and repo, so edits do not repeat the note.
fn remind_once(session_id: &str, repo: &Repo) -> bool {
    let Some(dir) = cache_dir().map(|dir| dir.join("reminded")) else {
        return false;
    };
    let marker = dir.join(format!(
        "{}-{}",
        path_key(Path::new(session_id)),
        path_key(&repo.root)
    ));
    if marker.exists() {
        return false;
    }
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(&marker, b"").is_ok()
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    at: u64,
    here: Here,
    role: Role,
}

/// The repo lookup and the user's role from cache, or from the server (and
/// then cached).
async fn cached_here(repo: &Repo, url: Option<&str>) -> Option<(Here, Role)> {
    let file = cache_dir()?
        .join("repos")
        .join(format!("{}.json", path_key(&repo.root)));
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let cached: Option<CacheEntry> = std::fs::read_to_string(&file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());
    if let Some(entry) = cached.filter(|e| now.saturating_sub(e.at) < CACHE_TTL.as_secs()) {
        return Some((entry.here, entry.role));
    }

    let config = Config::load(url).ok()?;
    let client = ApiClient::new(&config).ok()?;
    client.require_auth().await.ok()?;
    let ops = Ops::new(client);
    let (here, role) = tokio::join!(
        find(&ops, repo, &config.deploy_repos),
        user_role(&ops.client)
    );
    let entry = CacheEntry {
        at: now,
        here: here.ok()?,
        role: role?,
    };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&file, serde_json::to_vec(&entry).ok()?);
    Some((entry.here, entry.role))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_urls_compare_across_forms() {
        let expected = Some("github.com/ghentcdh/auto".to_string());
        assert_eq!(
            normalize_repo_url("git@github.com:GhentCDH/auto.git"),
            expected
        );
        assert_eq!(
            normalize_repo_url("https://github.com/GhentCDH/auto"),
            expected
        );
        assert_eq!(
            normalize_repo_url("https://github.com/GhentCDH/auto.git/"),
            expected
        );
        assert_eq!(
            normalize_repo_url("ssh://git@github.com:22/GhentCDH/auto.git"),
            expected
        );
        assert_eq!(normalize_repo_url("not a url"), None);
    }

    #[test]
    fn names_match_loosely() {
        assert!(names_match(
            "Archief Philippe Van Snick",
            "archief-philippe-van-snick"
        ));
        assert!(names_match("auto", "auto"));
        assert!(names_match("ghentcdh-wiki", "Wiki")); // substring, both >= 4 chars
        assert!(!names_match("api", "rapid")); // too short to match by substring
        assert!(!names_match("", "x"));
    }

    #[test]
    fn session_context_follows_role() {
        let wiki = Match {
            entity: "applications".into(),
            id: "1".into(),
            name: "wiki".into(),
            environment: None,
            matched_by: "repository_url".into(),
        };
        let here = Here {
            repo: Repo {
                root: PathBuf::from("/repo"),
                remote: None,
                name: "wiki".into(),
            },
            matches: vec![wiki.clone()],
            deploy_repos: vec![],
        };
        let viewer = session_context(&here, &wiki, Role::Viewer);
        assert!(viewer.contains("only read") && !viewer.contains("Keep auto in sync"));
        let editor = session_context(&here, &wiki, Role::Editor);
        assert!(editor.contains("Keep auto in sync"));
        assert_eq!(Role::deserialize(&json!("editor")).ok(), Some(Role::Editor));
    }

    #[test]
    fn deploy_files() {
        assert!(is_deploy_file("/repo/deploy/app.nomad.hcl"));
        assert!(is_deploy_file("/repo/docker-compose.prod.yml"));
        assert!(is_deploy_file("/repo/Caddyfile"));
        assert!(is_deploy_file("/repo/.github/workflows/release.yml"));
        assert!(is_deploy_file("/repo/Dockerfile.dev"));
        assert!(!is_deploy_file("/repo/src/main.rs"));
        assert!(!is_deploy_file("/repo/README.md"));
    }
}
