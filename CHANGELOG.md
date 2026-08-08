# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section here becomes that release's GitHub notes and the text shown in the app's
"what's new" panel — write it for a user, not for a commit log.

**Write the Vietnamese first.** Most people using this app read Vietnamese, and release
notes are read by users, not by the people who wrote the code. English goes in an
`### English` subsection at the end of the same release, as the secondary version.
Entries before 0.4.0 predate this and are English only.

## [0.4.0] - 2026-08-08

### Đã bỏ

- **Bỏ chế độ giữ phím để nói.** Trước đây có hai cách bắt đầu đọc — giữ một phím, hoặc
  bấm một lần rồi bấm lại — nên màn hình nào cũng phải giải thích cả hai, mà một phím
  hiện lên là `F9` thì chẳng nói được nó thuộc kiểu nào. Giờ chỉ còn một cách: bấm để
  bắt đầu, bấm lại để dán.
- Mục **Cài đặt → Giữ để nói** cũng không còn. File cài đặt của bản cũ vẫn đọc được
  bình thường, phần thiết lập cũ chỉ đơn giản là bị bỏ qua. Nếu đó là phím đọc duy nhất
  của bạn thì lần mở app kế tiếp phím mặc định sẽ được trả lại, để bạn không rơi vào
  cảnh không còn cách nào bắt đầu.

### Thay đổi

- Phím đọc mặc định giờ là `⌘/` trên macOS và `Control+Alt+D` trên Windows và Linux.
  Máy đang cài giữ nguyên phím bạn đã đặt.
- **Logo mới:** con nháy ở giữa dạng sóng âm được thay bằng cái micro. Sóng âm và màu
  giữ nguyên.

### Mới

- Video hướng dẫn 50 giây ở bước "thử một lần" của phần cài đặt lần đầu, lồng tiếng
  đúng theo ngôn ngữ bạn chọn cho app.
- Bước "thử một lần" giờ cho chữ hiện ra ngay trong app, nên để chứng minh phím tắt chạy
  được thì không phải mở thêm ứng dụng khác ra dán nữa.
- Phím huỷ giờ được hiện trong phần cài đặt và trong gợi ý phím tắt — phím này vốn đã
  có, chỉ là không chỗ nào nói ra.

### Sửa lỗi

- **Có lúc app đứng mãi ở trạng thái "đang xử lý".** Nếu kết nối tới dịch vụ nhận dạng
  giọng nói chết nửa chừng — máy ngủ, rớt Wi-Fi, VPN kết nối lại — app cứ chờ một câu
  trả lời không bao giờ tới: bảng điều khiển kẹt ở trạng thái bận, nút bị khoá, mà nút
  Huỷ thì thậm chí không hiện ra. Giờ mọi bước trao đổi đều có giới hạn thời gian, một
  lần đọc bị kẹt sẽ dừng sau 20 giây kèm thông báo lỗi rõ ràng, và nút Huỷ dùng được
  trong suốt lần đọc chứ không chỉ lúc đang thu.
- **Mở micro có thể làm treo cả cửa sổ**, không riêng phần đọc. Driver âm thanh bị kẹt
  hoặc tai nghe Bluetooth đang chuyển chế độ sẽ giữ luôn cái luồng vẽ cửa sổ và nhận
  phím tắt. Giờ quá 5 giây là dừng và báo micro không dùng được.
- Lúc khởi động, app hỏi từng thiết bị âm thanh toàn bộ khả năng của nó trước khi hiện
  bất cứ thứ gì — trên máy Windows nhiều thiết bị vào thì đó là vài giây màn hình trắng.
  Việc này giờ chạy nền.
- Kiểm tra cập nhật có thể kẹt ở "Đang kiểm tra…" hết phiên làm việc nếu kết nối tới
  GitHub bị treo. Giờ quá 15 giây là dừng.
- Video hướng dẫn chưa bao giờ hiện được trên bản đã cài — chính sách nội dung của app
  không cho tải media từ ngoài, mà điều này chỉ có bản đóng gói mới áp dụng.
- Gợi ý kiểm tra micro trên macOS lại chỉ sang phần cài đặt riêng tư của Windows.
- Bước "thử một lần" hiện khung kiểm tra micro thêm một lần nữa.
- Ghi chú phát hành chưa bao giờ tới được `latest.json`, nên panel "có gì mới" trong app
  trống trơn ở mọi bản cập nhật.

### English

#### Removed

- Hold-to-talk is gone. There were two ways to dictate — hold a key, or press it once and
  press it again — and having both meant every screen had to explain both, while a key
  shown as `F9` gave no clue which of the two it was. Dictation is now one key: press to
  start, press again to paste.
- Settings → Push to talk is gone with it. A settings file from an older build still
  loads; the old binding is simply ignored. If it was your only dictation key, the
  press-once default is restored on first launch so you are not left with no way in.

#### Changed

- The default dictation key is now `⌘/` on macOS and `Control+Alt+D` on Windows and
  Linux. Existing installs keep whatever key you already had.
- New app icon: the text cursor in the middle of the waveform is now a microphone. The
  waveform and the colour are unchanged.

#### Added

- A 50-second walkthrough video, narrated in the language the app is set to, in the
  "try it" step of first-run setup.
- First-run setup's "try it" step now shows the text land in the app itself, so proving
  the hotkey works no longer means opening a second app to paste into.
- The cancel key is now shown in setup and in the hotkey hints — it existed before but
  nothing on screen said so.

#### Fixed

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
