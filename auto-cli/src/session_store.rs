//! Persistence for the loopback login session token.
//!
//! Stored outside the main config, in the OS data directory
//! (`~/.local/share/auto/session` on Linux) with `0600` permissions. The
//! server expires it on its own schedule, at which point login runs again.

use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

use crate::config::APP_DIR;

fn path() -> Option<PathBuf> {
    dirs::data_dir().map(|base| base.join(APP_DIR).join("session"))
}

/// Load a previously stored session token, if any.
pub fn load() -> Option<String> {
    let value = fs::read_to_string(path()?).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Persist the session token with owner-only permissions.
pub fn save(token: &str) -> Result<()> {
    match path() {
        Some(path) => write_private(&path, token.as_bytes()),
        None => Ok(()),
    }
}

/// Remove the stored session token.
pub fn clear() {
    if let Some(path) = path() {
        let _ = fs::remove_file(path);
    }
}

/// Write a file readable only by its owner, creating parent directories.
pub fn write_private(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(content)?;
    }
    #[cfg(not(unix))]
    {
        fs::write(path, content)?;
    }
    Ok(())
}
