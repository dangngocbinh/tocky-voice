//! Whether this copy of the app is allowed to replace itself when it updates.
//!
//! On Windows and Linux the updater always may: the installer owns the files and no OS
//! permission is tied to the binary's identity.
//!
//! macOS is the one that has to be asked. Every permission the app depends on —
//! Accessibility above all, without which pasting silently stops working — is recorded
//! by the OS against the app's code signature. A Developer ID signature is stable
//! across releases, so an updated copy inherits the grants. An *unsigned* build is
//! identified by its cdhash instead, which changes with every build, so replacing it in
//! place would revoke Accessibility while System Settings still shows the toggle on:
//! the app appears to run and quietly pastes nothing. That is far worse than asking
//! someone to download an installer, so unsigned builds do not self-install.
//!
//! This is decided by looking at the running bundle rather than by a build-time flag,
//! so a local `cargo tauri build` and a release from CI each do the right thing with no
//! switch to remember.

/// Whether the updater may download and swap this app in place.
pub fn can_self_install() -> bool {
    #[cfg(target_os = "macos")]
    {
        developer_id_signed()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[cfg(target_os = "macos")]
fn developer_id_signed() -> bool {
    use std::sync::OnceLock;

    static SIGNED: OnceLock<bool> = OnceLock::new();
    *SIGNED.get_or_init(|| {
        let Some(bundle) = bundle_path() else {
            log::warn!("not running from an .app bundle; treating as unsigned");
            return false;
        };
        // `codesign -dv` writes its report to stderr, including when it succeeds.
        match std::process::Command::new("/usr/bin/codesign")
            .arg("-dv")
            .arg("--verbose=2")
            .arg("--")
            .arg(&bundle)
            .output()
        {
            Ok(output) => {
                let report = String::from_utf8_lossy(&output.stderr);
                let signed = has_developer_id(&report);
                log::info!("code signature: {}", if signed { "Developer ID" } else { "not Developer ID" });
                signed
            }
            Err(e) => {
                log::warn!("could not run codesign: {e}");
                false
            }
        }
    })
}

/// The `.app` directory the running executable lives inside, if any.
#[cfg(target_os = "macos")]
fn bundle_path() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(|path| path.to_path_buf())
}

/// True when `codesign`'s report shows a Developer ID Application certificate.
///
/// Ad-hoc signatures (`Signature=adhoc`, what an unsigned Tauri build gets) and
/// development certificates ("Apple Development") both have to fail here: neither
/// carries the stable identity that lets macOS keep the Accessibility grant across an
/// update, which is the entire reason for this check.
#[cfg(target_os = "macos")]
fn has_developer_id(report: &str) -> bool {
    report
        .lines()
        .any(|line| line.trim().starts_with("Authority=Developer ID Application"))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::has_developer_id;

    #[test]
    fn accepts_a_developer_id_signature() {
        let report = "Executable=/Applications/Tocky Voice.app/Contents/MacOS/tockyvoice\n\
             Identifier=pro.mecode.tockyvoice\n\
             Authority=Developer ID Application: Someone (TEAMID1234)\n\
             Authority=Developer ID Certification Authority\n\
             Authority=Apple Root CA\n";
        assert!(has_developer_id(report));
    }

    #[test]
    fn rejects_an_adhoc_signature() {
        let report = "Executable=/Applications/Tocky Voice.app/Contents/MacOS/tockyvoice\n\
             Identifier=pro.mecode.tockyvoice\n\
             Signature=adhoc\n";
        assert!(!has_developer_id(report));
    }

    /// A development certificate is not a distribution identity: it is tied to one
    /// machine's provisioning and does not survive a release the way Developer ID does.
    #[test]
    fn rejects_a_development_certificate() {
        let report = "Authority=Apple Development: Someone (TEAMID1234)\n\
             Authority=Apple Worldwide Developer Relations Certification Authority\n";
        assert!(!has_developer_id(report));
    }

    #[test]
    fn rejects_the_error_codesign_prints_for_an_unsigned_bundle() {
        assert!(!has_developer_id("/Applications/X.app: code object is not signed at all\n"));
    }
}
