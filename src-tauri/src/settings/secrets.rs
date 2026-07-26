//! Where API keys live.
//!
//! Two backends, chosen by the user:
//!
//! * **Local vault** (default) — a `0600` JSON file inside the app's `0700` data
//!   directory. Other user accounts on the machine cannot read it. Works identically
//!   on macOS, Windows and Linux with no system service required.
//! * **OS keychain** (opt-in) — Keychain / Credential Manager / Secret Service.
//!   Strictly stronger *when the app is code-signed*: the OS then authorises this
//!   specific binary and nothing else can read the keys.
//!
//! The default is the vault rather than the keychain, deliberately. An unsigned or
//! ad-hoc-signed macOS build has no stable identity, so the keychain re-prompts for
//! the **login password** on essentially every read. Conditioning a user to type their
//! account password into a recurring dialog is a worse security outcome than a file
//! only they can read — it is the exact shape of a credential-phishing attack. Users
//! running a properly signed build should turn the keychain on.
//!
//! Either way the plaintext never crosses the IPC boundary: the UI can write and probe
//! a key, never read one back.

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const SERVICE: &str = "pro.mecode.tockyvoice";
const VAULT_FILE: &str = "credentials.json";

/// Set once at startup so the accessors below stay callable as `secrets::get_key(name)`
/// from anywhere without threading an `AppHandle` through every layer.
static VAULT_PATH: OnceLock<PathBuf> = OnceLock::new();
static USE_KEYCHAIN: AtomicBool = AtomicBool::new(false);

/// Wires up the backend. Call once during setup, and again whenever the user changes
/// the preference so the switch takes effect without a restart.
pub fn configure(vault_dir: PathBuf, use_keychain: bool) {
    let _ = VAULT_PATH.set(vault_dir.join(VAULT_FILE));
    USE_KEYCHAIN.store(use_keychain, Ordering::Relaxed);
}

fn use_keychain() -> bool {
    USE_KEYCHAIN.load(Ordering::Relaxed)
}

fn vault_path() -> Result<&'static PathBuf> {
    VAULT_PATH
        .get()
        .context("credential store used before it was configured")
}

// ---------------------------------------------------------------- public API

pub fn set_key(account: &str, value: &str) -> Result<()> {
    // An empty submission means "forget this key".
    if value.trim().is_empty() {
        return delete_key(account);
    }
    if use_keychain() {
        keychain_entry(account)?
            .set_password(value)
            .context("writing to the OS keychain")
    } else {
        vault_write(account, Some(value))
    }
}

pub fn get_key(account: &str) -> Option<String> {
    let value = if use_keychain() {
        keychain_entry(account).ok()?.get_password().ok()
    } else {
        vault_read().ok()?.get(account).cloned()
    };
    value.filter(|v| !v.trim().is_empty())
}

pub fn has_key(account: &str) -> bool {
    get_key(account).is_some()
}

pub fn delete_key(account: &str) -> Result<()> {
    if use_keychain() {
        return match keychain_entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e).context("deleting from the OS keychain"),
        };
    }
    vault_write(account, None)
}

/// Moves every stored key to the other backend. Called when the preference changes so
/// switching does not silently look like all the keys vanished.
pub fn migrate_to(use_keychain_now: bool, accounts: &[&str]) -> Result<()> {
    if use_keychain() == use_keychain_now {
        return Ok(());
    }
    // Read under the old backend first…
    let existing: Vec<(String, String)> = accounts
        .iter()
        .filter_map(|a| get_key(a).map(|v| ((*a).to_string(), v)))
        .collect();

    USE_KEYCHAIN.store(use_keychain_now, Ordering::Relaxed);

    // …then write under the new one. A failure here leaves the old copy in place.
    for (account, value) in existing {
        if let Err(e) = set_key(&account, &value) {
            log::warn!("could not migrate the {account} key: {e:#}");
        }
    }
    Ok(())
}

/// Keychain account name for a speech provider.
pub fn stt_account(provider: &super::SttProviderKind) -> &'static str {
    match provider {
        super::SttProviderKind::Soniox => "soniox",
        super::SttProviderKind::Deepgram => "deepgram",
        super::SttProviderKind::AssemblyAi => "assemblyai",
    }
}

// ---------------------------------------------------------------- backends

fn keychain_entry(account: &str) -> Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, account).context(
        // The Linux backend needs a Secret Service provider running; on a minimal
        // desktop there may not be one, and the raw error does not say so.
        "could not reach the system credential store (macOS Keychain, Windows \
         Credential Manager, or a Linux Secret Service such as gnome-keyring / KWallet)",
    )
}

fn vault_read() -> Result<HashMap<String, String>> {
    let path = vault_path()?;
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Ok(HashMap::new());
    };
    let parsed: Map<String, Value> = serde_json::from_str(&raw).unwrap_or_default();
    Ok(parsed
        .into_iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
        .collect())
}

/// `None` deletes the entry. Rewrites the whole file each time — it holds a handful of
/// keys, so there is nothing to gain from partial updates.
fn vault_write(account: &str, value: Option<&str>) -> Result<()> {
    let path = vault_path()?;
    let mut keys = vault_read()?;
    match value {
        Some(v) => keys.insert(account.to_string(), v.to_string()),
        None => keys.remove(account),
    };
    let json = serde_json::to_string_pretty(&keys)?;
    crate::private_file::write(path, &json)
}
