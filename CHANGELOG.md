# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section here becomes that release's GitHub notes and the text shown in the app's
"what's new" panel — write it for a user, not for a commit log.

## [0.3.0] - 2026-07-30

### Added

- First-run setup now checks your microphone before anything else: pick an input and
  say something, and a level meter proves sound is actually arriving before you spend
  a speech-provider key finding out the hard way.
- Saving a speech-provider key now opens a real connection to check it works, right in
  setup — a typo, a revoked key, or an empty balance is reported immediately instead of
  showing up later as a dictation that produces nothing.

### Fixed

- Windows and Linux: the default push-to-talk key didn't work at all on a fresh
  install — it was a macOS-only binding. It's now `F9`, and existing installs are
  migrated to it automatically on next launch.
- Windows and Linux: the other default hotkeys (`Control+Alt+…`) collided with AltGr on
  many keyboard layouts; they're now `Control+Shift+…`.
- Audio input devices that deliver less common sample formats — common on USB audio
  interfaces and "Line in" codecs under Windows — are now supported instead of failing
  to open.
- A microphone that opens but never picks up any sound (muted, blocked by Windows
  privacy settings, or the wrong device) now says so, instead of the dictation panel
  silently closing after a couple of seconds.
- Windows: saving settings no longer logs a spurious autostart warning on every
  keystroke.

## [0.2.1] - 2026-07-28

### Fixed

- Release pipeline: the manifest-verification step needs `contents: write` to read a
  draft release, not `contents: read` — GitHub's API returns 403 otherwise. This release
  exists to prove the v0.2.0 → v0.2.1 update path actually works end to end.

## [0.2.0] - 2026-07-28

### Added

- In-app update check: Tocky Voice checks GitHub for a newer release once per launch
  (toggle in Settings) and shows what changed before you install it.
- Windows and Linux (AppImage) install updates automatically and relaunch. macOS opens
  the release page instead, since installing in place on an unsigned build would break
  the Accessibility permission — see the README for why.
- The running version is now shown in Settings → About.
- README: documented the `xattr -cr` fix for macOS's "app is damaged" Gatekeeper message.

## [0.1.0] - 2026-07-26

### Added

- First public release: hold a key, talk, and the transcript is pasted into whatever
  app has focus.
- Speech-to-text via Deepgram, Soniox or AssemblyAI; optional AI cleanup with Claude,
  OpenAI, Gemini, DeepSeek, Qwen, Kimi, GLM, MiniMax, Groq, xAI, OpenRouter, or a local
  Ollama.
- Four built-in modes — Raw, Clean, Prompt, Email — each with its own hotkey.
- Vietnamese and English interface.
- macOS, Windows and Linux installers.
