//! Persistence for the loopback login session token.
//!
//! Stored outside the main config, in the OS data directory
//! (`~/.local/share/auto-tui/session` on Linux) with `0600` permissions. The
//! server expires it on its own schedule, at which point login runs again.

use std::fs;
use std::path::PathBuf;

use color_eyre::eyre::Result;

fn path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("auto-tui").join("session"))
}

/// Load a previously stored session token, if any.
pub fn load() -> Option<String> {
    let path = path()?;
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// Persist the session token with owner-only permissions.
pub fn save(token: &str) -> Result<()> {
    let Some(path) = path() else {
        return Ok(());
    };
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
            .open(&path)?;
        file.write_all(token.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        fs::write(&path, token)?;
    }
    Ok(())
}

/// Remove the stored session token.
pub fn clear() {
    if let Some(path) = path() {
        let _ = fs::remove_file(path);
    }
}
