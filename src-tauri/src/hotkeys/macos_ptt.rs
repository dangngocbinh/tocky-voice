//! Hold-a-modifier push-to-talk on macOS.
//!
//! A bare modifier such as Right Option can't be expressed as an accelerator, so the
//! standard global-shortcut plugin can't bind it. Instead we attach a listen-only
//! CGEventTap for `flagsChanged` on a dedicated thread with its own run loop.
//! Listen-only means we never swallow the key — Option keeps working normally.

use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::settings::ModifierKey;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

/// Both the event tap and synthesized paste keystrokes need Accessibility permission,
/// so this one check covers the two features that can silently do nothing without it.
pub fn has_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Same check, but shows the system prompt with its "Open System Settings" shortcut
/// when the permission is missing. Without it the app just silently does nothing —
/// no paste, no push-to-talk — which is impossible for a user to diagnose.
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

/// Keeps the unused-import warning away on the `CFTypeRef` alias used above.
const _: Option<CFTypeRef> = None;

/// Virtual keycodes for the modifiers we can bind. `flagsChanged` reports which
/// physical key moved, which is how left and right Option are told apart.
fn keycode(key: ModifierKey) -> i64 {
    match key {
        ModifierKey::LeftOption => 58,
        ModifierKey::RightOption => 61,
        ModifierKey::RightCommand => 54,
        ModifierKey::Fn => 63,
    }
}

/// Starts the listener thread. `on_change(true)` fires on press, `on_change(false)`
/// on release. Returns a flag that stops the run loop when set to false.
pub fn spawn_listener<F>(key: ModifierKey, on_change: F) -> Arc<AtomicBool>
where
    F: Fn(bool) + Send + 'static,
{
    let running = Arc::new(AtomicBool::new(true));
    let thread_running = running.clone();
    let target = keycode(key);

    std::thread::Builder::new()
        .name("fvt-ptt-listener".into())
        .spawn(move || {
            // Tracks the last observed state so we emit one event per transition
            // rather than one per flagsChanged message. The tap callback is `Fn`,
            // so this has to be shared interior state rather than a captured `mut`.
            let held = AtomicBool::new(false);

            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![CGEventType::FlagsChanged],
                move |_proxy, _event_type, event| {
                    let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                    if code == target {
                        // On a flagsChanged for our key, a non-empty flag set means
                        // it went down; an empty one means every modifier was released.
                        let now_held = !event.get_flags().is_empty();
                        if held.swap(now_held, Ordering::Relaxed) != now_held {
                            on_change(now_held);
                        }
                    }
                    None
                },
            );

            let Ok(tap) = tap else {
                log::error!(
                    "could not create the push-to-talk event tap — grant Accessibility \
                     permission to Tocky Voice and restart the app"
                );
                return;
            };

            let Ok(source) = tap.mach_port.create_runloop_source(0) else {
                log::error!("could not create a run loop source for the push-to-talk tap");
                return;
            };

            let run_loop = CFRunLoop::get_current();
            unsafe { run_loop.add_source(&source, kCFRunLoopCommonModes) };
            tap.enable();

            // Wake periodically so a cleared `running` flag can stop the thread.
            // The source is registered in the *common modes* set, but the loop has to
            // be run in a concrete mode — `kCFRunLoopCommonModes` is a set marker and
            // is rejected by CFRunLoopRunInMode. Default mode is part of that set, so
            // the source still fires.
            while thread_running.load(Ordering::Relaxed) {
                CFRunLoop::run_in_mode(
                    unsafe { kCFRunLoopDefaultMode },
                    std::time::Duration::from_millis(250),
                    false,
                );
            }
            run_loop.remove_source(&source, unsafe { kCFRunLoopCommonModes });
        })
        .ok();

    running
}
