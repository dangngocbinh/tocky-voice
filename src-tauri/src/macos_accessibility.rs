//! The macOS Accessibility permission check.
//!
//! Synthesizing the paste keystroke is privileged on macOS, and without the permission
//! it fails silently — the dictation completes, nothing appears, and there is no error
//! to go on. Both the startup prompt and the settings permission banner ask here.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

pub fn has_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Same check, but shows the system prompt with its "Open System Settings" shortcut
/// when the permission is missing.
///
/// macOS shows the prompt at most once per app per login session; after that the call
/// is equivalent to [`has_accessibility_permission`].
pub fn prompt_for_accessibility_permission() -> bool {
    unsafe {
        let options = CFDictionary::from_CFType_pairs(&[(
            CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt).as_CFType(),
            CFBoolean::true_value().as_CFType(),
        )]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as CFDictionaryRef)
    }
}
