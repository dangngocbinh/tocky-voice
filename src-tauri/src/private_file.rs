//! Writing files that only the current user can read.
//!
//! This app persists dictation transcripts and the audio recordings behind them. The
//! default umask leaves those world-readable (0644), which on a shared machine means
//! any other local account can read everything the user has ever dictated. Every write
//! of user content goes through here instead.

use anyhow::{Context, Result};
use std::path::Path;

#[cfg(unix)]
const FILE_MODE: u32 = 0o600;
#[cfg(unix)]
const DIR_MODE: u32 = 0o700;

/// Creates a directory owner-only. Existing directories are tightened too, so users
/// upgrading from an earlier build stop leaking their old recordings.
pub fn create_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .with_context(|| format!("creating {}", path.display()))?;
    restrict_dir(path);
    Ok(())
}

/// Writes a file owner-only.
///
/// The permissions are applied *before* the contents on Unix — creating the file empty
/// and chmod-ing it first closes the window where a world-readable file briefly holds
/// real data.
pub fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(FILE_MODE)
            .open(path)
            .with_context(|| format!("opening {}", path.display()))?;
        file.write_all(contents.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        // `mode` only applies when the file is newly created; an existing file keeps
        // whatever it had, so tighten it explicitly.
        restrict_file(path);
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        // Windows inherits the ACL of the parent directory, which for the per-user
        // AppData tree is already restricted to the account that owns it.
        std::fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
    }
}

/// Tightens permissions on a file the app did not create through [`write`] —
/// notably the WAV recordings, which are written by an encoder that owns the handle.
pub fn restrict_file(path: &Path) {
    #[cfg(unix)]
    set_mode(path, FILE_MODE);
    #[cfg(not(unix))]
    let _ = path;
}

fn restrict_dir(path: &Path) {
    #[cfg(unix)]
    set_mode(path, DIR_MODE);
    #[cfg(not(unix))]
    let _ = path;
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)) {
        log::warn!(
            "could not restrict permissions on {}: {e} — it may be readable by other \
             users on this machine",
            path.display()
        );
    }
}
