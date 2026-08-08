# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section here becomes that release's GitHub notes and the text shown in the app's
"what's new" panel — write it for a user, not for a commit log.

## [0.4.0] - 2026-08-08

### Removed

- Hold-to-talk is gone. There were two ways to dictate — hold a key, or press it once and
  press it again — and having both meant every screen had to explain both, while a key
  shown as `F9` gave no clue which of the two it was. Dictation is now one key: press to
  start, press again to paste.
- Settings → Push to talk is gone with it. A settings file from an older build still
  loads; the old binding is simply ignored. If it was your only dictation key, the
  press-once default is restored on first launch so you are not left with no way in.

### Changed

- The default dictation key is now `⌘/` on macOS and `Control+Alt+D` on Windows and
  Linux. Existing installs keep whatever key you already had.
- New app icon: the text cursor in the middle of the waveform is now a microphone. The
  waveform and the colour are unchanged.

### Added

- A 50-second walkthrough video, narrated in the language the app is set to, in the
  "try it" step of first-run setup.
- First-run setup's "try it" step now shows the text land in the app itself, so proving
  the hotkey works no longer means opening a second app to paste into.
- The cancel key is now shown in setup and in the hotkey hints — it existed before but
  nothing on screen said so.

### Fixed

- A dictation could hang on "transcribing" forever. If the connection to the speech
  provider half-died mid-take — sleeping laptop, dropped wifi, a VPN reconnecting — the
  app waited on an answer that was never coming: the panel stayed busy, the button
  stayed disabled, and Cancel wasn't even on screen. Every step of that conversation is
  now on a clock, a stuck take gives up after 20 seconds with a real error, and Cancel
  is available for the whole take instead of only while recording.
- Opening the microphone could freeze the entire window, not just the dictation. A
  wedged audio driver or a Bluetooth headset switching profiles held up the thread that
  draws the window and answers the hotkeys. Opening now gives up after 5 seconds and
  says the microphone is unavailable.
- Startup asked every audio device for its full capability list before showing anything,
  which on Windows machines with several inputs meant seconds of blank window. That now
  happens in the background.
- The update check could sit on "Checking…" for the rest of the session if the
  connection to GitHub stalled. It now gives up after 15 seconds.
- The walkthrough video never appeared in an installed build — the app's content policy
  allowed no remote media, something only a packaged build enforces.
- The mic-check hint on macOS pointed at Windows privacy settings.
- The "try it" step showed the microphone box a second time.
- Release notes never reached `latest.json`, so the app's "what's new" panel was blank
  for every update.

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
