# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section here becomes that release's GitHub notes and the text shown in the app's
"what's new" panel — write it for a user, not for a commit log.

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
