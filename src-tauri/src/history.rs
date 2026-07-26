//! Dictation history: every take is kept so a lost or mangled result can be recovered
//! and re-copied. Both the raw transcript and the AI-refined text are stored, because
//! when cleanup goes wrong the raw version is usually the one you want back.

use crate::settings::HistorySettings;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const HISTORY_FILE: &str = "history.json";
const AUDIO_DIR: &str = "recordings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub mode_id: String,
    pub mode_name: String,
    /// Exactly what the speech provider returned.
    pub raw_text: String,
    /// What actually got pasted — equals `raw_text` when the mode skips AI cleanup.
    pub final_text: String,
    pub duration_secs: f32,
    pub stt_provider: String,
    /// Absolute path to the WAV, when audio retention is on and the file still exists.
    pub audio_path: Option<String>,
}

pub fn history_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .context("no app data dir available")?;
    crate::private_file::create_dir(&dir)?;
    Ok(dir.join(HISTORY_FILE))
}

pub fn audio_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .context("no app data dir available")?
        .join(AUDIO_DIR);
    crate::private_file::create_dir(&dir)?;
    Ok(dir)
}

pub fn load(app: &AppHandle) -> Vec<HistoryEntry> {
    let Ok(path) = history_path(app) else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        log::error!("history.json unreadable ({e}); starting a fresh list");
        Vec::new()
    })
}

fn save(app: &AppHandle, entries: &[HistoryEntry]) -> Result<()> {
    let path = history_path(app)?;
    // Transcripts are sensitive: everything the user has ever dictated lives here.
    crate::private_file::write(&path, &serde_json::to_string_pretty(entries)?)
}

/// Prepends an entry, then applies both retention policies: the entry cap and the
/// audio age limit. Audio files belonging to evicted entries are deleted with them,
/// so the recordings dir can't outgrow the history it belongs to.
pub fn append(app: &AppHandle, entry: HistoryEntry, settings: &HistorySettings) -> Result<()> {
    let mut entries = load(app);
    entries.insert(0, entry);

    for evicted in entries.iter().skip(settings.max_entries) {
        remove_audio(evicted);
    }
    entries.truncate(settings.max_entries);

    prune_expired_audio(&mut entries, settings.audio_retention_days);
    save(app, &entries)
}

/// Clears audio for entries past the retention window; the text is kept.
fn prune_expired_audio(entries: &mut [HistoryEntry], retention_days: i64) {
    if retention_days <= 0 {
        return;
    }
    let cutoff = Utc::now() - Duration::days(retention_days);
    for entry in entries.iter_mut().filter(|e| e.created_at < cutoff) {
        if entry.audio_path.is_some() {
            remove_audio(entry);
            entry.audio_path = None;
        }
    }
}

fn remove_audio(entry: &HistoryEntry) {
    if let Some(path) = entry.audio_path.as_deref() {
        if let Err(e) = std::fs::remove_file(Path::new(path)) {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("could not delete recording {path}: {e}");
            }
        }
    }
}

pub fn delete(app: &AppHandle, id: &str) -> Result<()> {
    let mut entries = load(app);
    if let Some(pos) = entries.iter().position(|e| e.id == id) {
        remove_audio(&entries[pos]);
        entries.remove(pos);
    }
    save(app, &entries)
}

pub fn clear(app: &AppHandle) -> Result<()> {
    for entry in load(app) {
        remove_audio(&entry);
    }
    save(app, &[])
}
