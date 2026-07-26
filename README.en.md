# Tocky Voice · Tốc Ký

**English** · [Tiếng Việt](README.md)

### Don't type. Just talk.

Hold a key anywhere on your computer, talk, and the text lands in whatever app you were
typing in — transcribed by a realtime speech API and optionally cleaned up by an LLM first.

Free and open source. You bring your own API keys, so there is no subscription and no
middleman: your audio goes from your machine straight to the provider you chose.

Built with Tauri v2 (Rust core + React UI). Interface in **English and Tiếng Việt**.

---

## Install

### Option 1 — Download and run (no tools needed)

Grab the file for your system from the [**Releases page**](../../releases/latest):

| System | File | Then |
| --- | --- | --- |
| macOS, Apple Silicon (M1–M4) | `..._aarch64.dmg` | Open the .dmg, drag the app to Applications |
| macOS, Intel | `..._x64.dmg` | Same |
| Windows 10/11 | `..._x64-setup.exe` or `..._x64_en-US.msi` | Run it |
| Linux | `..._amd64.AppImage` | `chmod +x` the file, then run it |
| Linux (Debian/Ubuntu) | `..._amd64.deb` | `sudo dpkg -i <file>.deb` |

**The app is not signed with a paid developer certificate**, so each OS will warn you the
first time. That is expected for open-source software, and here is how to get past it:

- **macOS** — right-click the app → **Open** → **Open**. (Double-clicking will just refuse.)
- **Windows** — SmartScreen shows a blue box → **More info** → **Run anyway**.

### Option 2 — Let a coding agent install it

Paste this into Claude Code, Cursor, Codex, or any agent with shell access:

```
Clone https://github.com/dangngocbinh/tocky-voice and build it for my machine.

  git clone https://github.com/dangngocbinh/tocky-voice
  cd tocky-voice
  pnpm install
  pnpm tauri build

Prerequisites: Node 20+, pnpm, and the Rust toolchain (https://rustup.rs).
On Ubuntu/Debian also:
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
                   libasound2-dev patchelf build-essential libxdo-dev libssl-dev

The installer lands in src-tauri/target/release/bundle/. Install it, launch the app,
and follow the 4-step setup it shows on first run.
```

> Replace `dangngocbinh` with the actual GitHub owner if you forked or renamed the repo.

### Option 3 — Build it yourself

```sh
pnpm install
pnpm tauri build          # installers in src-tauri/target/release/bundle/
```

---

## First run

The app walks you through setup in four steps, and you can re-run it any time from
**Settings → Run setup again**:

1. **Language** — English or Tiếng Việt (or follow your system).
2. **Speech service** — pick one and paste a free API key. The app does nothing without this.
3. **Permission** (macOS only) — see below.
4. **Try it** — say a sentence and watch the words appear, then go use it for real.

### Getting a speech key (free)

| Provider | Free credit | Streaming price | Pick it when |
| --- | --- | --- | --- |
| **Deepgram** — *start here* | **$200, no card** (~690 hours) | ~$0.29/hour | You want to be up and running without spending anything |
| **Soniox** — *best for Vietnamese* | None | **$0.12/hour — cheapest** | You mix Vietnamese and English in one sentence |
| AssemblyAI | $50, no card | $0.15/hour | You mostly speak English (no Vietnamese in streaming yet) |

Soniox is the most accurate but bills from the first minute — and is the cheapest per
hour in exchange. Deepgram's $200 makes it the sensible place to start; switch to Soniox
once mixed Vietnamese-English sentences start coming out wrong. Changing provider takes
seconds in Settings.

Sign-up links are built into the app, next to the empty key field.

### macOS: Accessibility permission

macOS blocks one app from typing into another unless you allow it. Without this
permission your words reach the clipboard but never land in the app you are writing in.

**System Settings → Privacy & Security → Accessibility** → switch on *Tocky Voice*.

> **Already switched on but the app still says it is missing?** macOS ties the permission
> to the exact copy of the app it was granted to, so a rebuilt or updated app needs it
> again — and the old switch keeps showing as *on*, which is misleading. Select the row,
> click **−**, then **+** and add the app back. Toggling it off and on does not work.

---

## Using it

Press the hotkey in any app, talk, press it again. The text is pasted where your cursor is.

| Action | Default |
| --- | --- |
| Start / stop | `Control+Alt+D` |
| Hold to talk | Right Option *(macOS only — bind a key combination on Windows/Linux)* |
| Cancel the take | `Control+Alt+X` |
| Next mode | `Control+Alt+M` |

All of these are configurable in **Settings → Shortcuts**.

### Modes

A mode is a prompt plus a delivery rule, so the same voice input can become different
things. Four ship by default, and you can add your own with a dedicated hotkey each:

- **Raw** — no AI, paste the transcript immediately (fastest, ~0 extra latency)
- **Clean** — fix spelling, punctuation and filler words; keep English technical terms intact
- **Prompt** — rewrite into a structured request for a coding agent
- **Email** — rewrite as a formal message

### AI cleanup (optional)

Anthropic Claude, OpenAI, Google Gemini, DeepSeek, Qwen, Moonshot Kimi, Zhipu GLM,
MiniMax, Groq, xAI, OpenRouter, local Ollama, or any OpenAI-compatible endpoint. The model
list is fetched live from whichever provider you pick, so a model released today is
selectable today.

Measured on the *Clean* mode, Vietnamese with English terms:

| Provider / model | Time |
| --- | --- |
| DeepSeek `deepseek-v4-flash` | ~1.1 s |
| OpenAI `gpt-4.1-mini` | ~1.7 s |

If a provider is slow, check whether the model is a *reasoning* model: those spend seconds
producing hidden reasoning nobody reads before answering. DeepSeek V4 went from 10.8 s to
1.9 s once that was switched off, which the app now does for you.

---

## Platform support

The app is developed and used daily on macOS. CI compiles and tests all three platforms on
every push, but **Windows and Linux have not been through hands-on testing yet** — treat
them as beta and please report what breaks.

| | macOS | Windows | Linux |
| --- | --- | --- | --- |
| Dictate → paste | ✅ | ✅ built, untested | ⚠️ see below |
| Global hotkeys | ✅ | ✅ built, untested | ⚠️ see below |
| Hold a bare modifier to talk | ✅ | ❌ use a key combination | ❌ use a key combination |
| Return focus to the exact app you started from | ✅ | not implemented — hiding the overlay restores focus in practice | same |
| Key file permissions | `0600` | inherits the per-user AppData ACL | `0600` |

**Linux caveat:** synthesizing keystrokes and grabbing global hotkeys are restricted under
Wayland in ways they are not under X11. If the hotkey or the paste does nothing on a
Wayland session, an X11 session is the workaround. This is untested either way.

---

## Where your API keys are kept

By default in `credentials.json` — mode `0600`, inside a `0700` data directory, so no other
account on the machine can read it. Never in `settings.json`, and the UI can only ask
*whether* a key exists; there is no command that reads one back.

**Settings → Providers** can switch this to the OS keychain (Keychain / Credential Manager /
Secret Service), which is stronger because the OS then authorises this exact binary. Turn it
on **once you are running a code-signed build**. On an unsigned macOS build the keychain
re-prompts for your *login password* on every read — a tool that keeps asking for your
account password is training you for a phishing attack, which is why it is not the default.
Switching either way migrates existing keys automatically.

## Data on disk

macOS: `~/Library/Application Support/pro.mecode.tockyvoice/`
Windows: `%APPDATA%\pro.mecode.tockyvoice\`
Linux: `~/.config/pro.mecode.tockyvoice/`

- `settings.json` — everything except credentials
- `credentials.json` — API keys, owner-only
- `history.json` — raw transcript + polished text per dictation
- `recordings/` — the WAV for each take, pruned by the retention setting

Nothing is sent anywhere except the speech provider and the LLM provider you configured.
There is no telemetry. **Teaching a class?** Consider turning off *Save recordings* in
Settings → History before handing this to students.

---

## How it works

```
hotkey ─▶ mic capture (cpal, 16 kHz mono) ─▶ WebSocket to STT provider ─▶ transcript
                     │                                                       │
                     └─▶ floating overlay: level meter + live text           ▼
                                                              AI cleanup (per-mode prompt)
                                                                             │
                                                                             ▼
                                                        clipboard + ⌘V into the focused app
```

The overlay never takes keyboard focus. That is the whole trick: the app you were typing in
stays frontmost, so the synthesized paste keystroke goes there and not to us.

## Development

```sh
pnpm install
pnpm tauri dev                             # hot reload
pnpm tauri build --debug --bundles app     # a .app you can grant permissions to
```

Permissions are tied to the binary's path and signature, so use the bundled app for
anything involving the microphone or pasting — a bare `cargo run` binary will be denied.

> Running the app **from a terminal on macOS gives a misleading result**: a process started
> from a terminal inherits the terminal's Accessibility trust, so it reports the permission
> as granted when the same bundle opened from the Dock does not. Always verify from the Dock.

## Tests

```sh
cd src-tauri
cargo test --lib                           # unit tests, no network
```

Live round-trip tests hit the real APIs, so they are `#[ignore]`d. `--test-threads=1` is not
optional: free tiers cap concurrent streaming sessions, and parallel runs fail for reasons
that have nothing to do with the code.

```sh
cd src-tauri
FVT_TEST_WAV=/path/to/16k-mono.wav \
SONIOX_API_KEY=... DEEPGRAM_API_KEY=... ASSEMBLYAI_API_KEY=... \
cargo test --test stt_provider_round_trip -- --ignored --nocapture --test-threads=1

FVT_LLM_PRESET=deepseek FVT_LLM_MODEL=deepseek-v4-flash FVT_LLM_KEY=sk-... \
FVT_SETTINGS="$HOME/Library/Application Support/pro.mecode.tockyvoice/settings.json" \
cargo test --test mode_round_trip -- --ignored --nocapture --test-threads=1
```

Making a Vietnamese fixture on macOS:

```sh
say -v Linh -o sample.aiff "Xin chào, hôm nay tôi sẽ deploy cái API này lên server."
afconvert -f WAVE -d LEI16@16000 -c 1 sample.aiff sample-vi.wav
```

## Releasing

Push a tag and CI builds every platform and opens a draft release:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

---

## License

[MIT](LICENSE) — use it, change it, ship it, sell it. The one condition is that the
copyright line stays in any copy or substantial portion of the code, so the work keeps
pointing back to where it came from.

Forks and student projects are welcome and need no permission.

### About the name and the logo

The MIT licence covers the **code**. It does not grant rights to the names *Tocky Voice*
or *Tốc Ký*, the *ME Code* name, or their logos — those are handled by trademark, not by a
copyright licence, and they are not part of the grant above.

In practice: build whatever you like on this, but give your version its own name and its
own icon rather than shipping something that looks like it came from us. Saying "based on
Tocky Voice by ME Code" is fine and appreciated.

## Credits

Built by [ME Code](https://mecode.pro) — AI Automation Academy & Product Lab.
