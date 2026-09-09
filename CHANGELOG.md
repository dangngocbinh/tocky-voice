# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section here becomes that release's GitHub notes and the text shown in the app's
"what's new" panel — write it for a user, not for a commit log.

**Write the Vietnamese first.** Most people using this app read Vietnamese, and release
notes are read by users, not by the people who wrote the code. English goes in an
`### English` subsection at the end of the same release, as the secondary version.
Entries before 0.4.0 predate this and are English only.

## [0.5.1] - 2026-09-09

### Sửa lỗi

- **Linux: cài xong app không mở lên được.** Gói `.deb` và `.rpm` cài trót lọt, nhưng
  chạy lên là tắt ngay với dòng `libxdo.so.3: cannot open shared object file`. Gói không
  khai báo hai thư viện app cần lúc chạy — `libxdo` (thứ bơm phím để dán chữ ra) và ALSA
  (thu âm) — nên trình quản lý gói không biết đường cài kèm. Máy nào tình cờ đã có sẵn thì
  chạy được, máy sạch thì chết. Giờ cả hai nằm trong danh sách phụ thuộc của gói.
- **Không kết nối được nhận diện giọng nói khi mạng có IPv6 hỏng.** Bấm Test key thì báo
  nhà cung cấp không trả lời sau 10 giây, trong khi trình duyệt trên cùng máy đó vẫn vào
  mạng bình thường — nên trông như key sai hoặc dịch vụ chết. Máy chủ trả về cả địa chỉ
  IPv6 lẫn IPv4; app thử lần lượt từ trên xuống, mà nếu đường IPv6 của mạng bị chặn kiểu
  im lặng — gói tin bị nuốt, không hề báo lỗi về — thì app ngồi đợi hết giờ ngay ở địa chỉ
  đầu tiên và không bao giờ chạm tới địa chỉ IPv4 vẫn đang tốt. Giờ app chạy đua các địa
  chỉ đúng cách trình duyệt vẫn làm: mỗi địa chỉ được 250 mili-giây trước khi địa chỉ kế
  tiếp vào đua cùng, ai trả lời trước thì dùng. Một họ địa chỉ chết chỉ còn tốn đúng 250
  mili-giây thay vì cả phiên làm việc.
- **Linux/Wayland: không phím tắt nào hoạt động.** App đăng ký phím tắt toàn cục theo
  kiểu X11, mà Wayland thì không giao phím cho kiểu đăng ký đó. Tệ hơn, lệnh đăng ký vẫn
  báo thành công, nên trong log mọi thứ trông hoàn toàn bình thường trong khi không phím
  nào bao giờ nổ. Hầu hết máy Linux giờ chạy Wayland mặc định. Giờ app nhận lệnh từ dòng
  lệnh — `tockyvoice --toggle`, `--cancel`, `--next-mode`, `--read`, `--read-voice`,
  `--mode=<tên>` — và chuyển thẳng vào bản đang chạy. Bạn vào phần cài đặt bàn phím của
  môi trường desktop, thêm phím tắt trỏ tới lệnh đó; đường này do chính compositor bắt
  phím nên chạy được trên Wayland.

### English

### Fixed

- **Linux: the app installed fine and then would not start.** The `.deb` and `.rpm`
  packages installed without complaint, then died on launch with
  `libxdo.so.3: cannot open shared object file`. The packages never declared two
  libraries the app needs at runtime — `libxdo`, which synthesizes the keystrokes that
  paste your text, and ALSA, which records it — so the package manager had no reason to
  pull them in. Machines that happened to have them already worked; clean ones did not.
  Both are now declared as dependencies.
- **Speech recognition would not connect on a network with broken IPv6.** Testing the
  key reported that the provider did not answer within 10 seconds, while a browser on
  the same machine reached the internet perfectly well — so it looked like a bad key or
  a dead service. The provider resolves to both IPv6 and IPv4 addresses, and the app
  tried them strictly in order. Where a network blackholes IPv6 — packets dropped, no
  error ever sent back — the app spent its entire budget waiting on the first address
  and never reached the working IPv4 one. Addresses are now raced the way browsers race
  them: each gets a 250 ms head start before the next joins in, and the first to answer
  wins. A dead address family now costs 250 ms instead of the whole session.
- **Linux/Wayland: none of the hotkeys worked.** The app claimed its global hotkeys the
  X11 way, and Wayland does not deliver keys to that kind of claim. Worse, the claim
  still reported success, so the logs looked entirely healthy while not one hotkey ever
  fired. Wayland is now the default on most Linux desktops. The app now accepts the
  actions on its command line — `tockyvoice --toggle`, `--cancel`, `--next-mode`,
  `--read`, `--read-voice`, `--mode=<id>` — and relays them to the running instance.
  Bind a combination to one of those in your desktop's own keyboard settings: there the
  compositor itself catches the key, which is the part that works on Wayland.

## [0.5.0] - 2026-09-04

### Mới

- **Đọc văn bản bôi đen.** Bôi đen chữ ở bất kỳ đâu, bấm `⌘⇧/` (macOS) hay
  `Control+Shift+R` (Windows/Linux), app đọc cho bạn nghe. Mặc định **tắt** — bật ở tab
  "Đọc to" mới trong Cài đặt.
- Bốn chế độ đọc: nguyên văn, tóm tắt, giải thích dễ hiểu, dịch sang tiếng Việt. Mỗi chế
  độ gán được phím tắt riêng.
- **Ra lệnh bằng giọng nói:** bôi đen rồi bấm `⌘⇧.`, nói yêu cầu ("tóm tắt rồi đọc cho
  tôi nghe"), bấm lại — AI làm theo và đọc kết quả lên, không cần vào Cài đặt chọn chế độ.
  Dùng chung API key bạn đã lưu, không phải đăng ký thêm gì.
- Bốn nhà cung cấp giọng đọc: Soniox (mặc định, rẻ nhất), Gemini, OpenAI, ElevenLabs
  (giọng đẹp nhất, cần key riêng). Đổi được ngay trong tab "Đọc to", có nút "Nghe thử".
- Thanh điều khiển nhỏ nổi trên màn hình lúc đang đọc — tạm dừng, dừng, kéo đi chỗ khác
  được, nhớ vị trí cho lần sau.
- Thêm một màn ở cuối phần cài đặt lần đầu để giới thiệu tính năng này — bỏ qua được
  bằng một cú bấm.
- **Đọc từ clipboard khi không lấy được chữ bôi đen.** Một số app (rõ nhất là Terminal,
  iTerm) không cho app khác copy hộ, nên bấm phím tắt xong thanh điều khiển hiện lên mà
  không có gì để đọc. Giờ nếu trong clipboard đang có sẵn nội dung, app sẽ hỏi "đọc nội
  dung đang có trong clipboard nhé?" — bạn tự `⌘C`/`Ctrl+C` trước rồi bấm phím tắt, mất
  thêm một thao tác nhưng nghe được ở mọi chỗ.

### Sửa lỗi

- **Windows: app kẹt vĩnh viễn ở màn hình "Loading".** Cửa sổ chính mở ra đen thui, chỉ
  có chữ "Loading", không bao giờ vào được giao diện — trong khi phím tắt vẫn chạy và
  micro vẫn nhận. Cửa sổ được tạo *trước* khi phần backend kịp dựng xong kho cài đặt, nên
  câu hỏi đầu tiên của giao diện ("cài đặt của tôi đâu?") bị trả lời là lỗi, và giao diện
  nuốt lỗi đó rồi ngồi đợi mãi một câu trả lời không bao giờ tới. Chạy nhanh hay chậm là
  do may rủi — WebView2 Runtime 152 mở trang sớm hơn nên máy nào cập nhật lên bản đó là
  dính chắc. Giờ cửa sổ chỉ được tạo sau khi backend đã sẵn sàng; và nếu vẫn hỏng vì lý
  do khác, app tự hỏi lại vài lần rồi hiện hẳn lỗi kèm nút "Thử lại" thay vì đứng im.
- **Không còn xoá mất ảnh đang có trong clipboard.** Trước khi lấy chữ bôi đen, app dọn
  clipboard để khỏi đọc nhầm nội dung cũ. Nhưng nếu bạn vừa copy một tấm ảnh hay một
  file, thứ đó bị xoá luôn và không lấy lại được. Giờ app chỉ dọn khi trong clipboard
  đang là chữ — ảnh và file được để nguyên.

### An ninh

- Văn bản bôi đen được gửi tới nhà cung cấp AI và đọc mà bạn chọn ở tab "Đọc to" —
  không gửi đi đâu khác, không lưu lại lịch sử các lần đọc.

### English

- **Read selected text aloud.** Highlight text anywhere, press `⌘⇧/` (macOS) or
  `Control+Shift+R` (Windows/Linux), and the app reads it back to you. Off by default —
  turn it on from the new "Read aloud" tab in Settings.
- **Reads the clipboard when the selection can't be grabbed.** Some apps — terminals
  most of all — refuse to hand over a highlighted passage, so the player came up with
  nothing to read. It now offers to read whatever is already on the clipboard instead:
  copy by hand, press the shortcut, take the offer.
- Four read modes: verbatim, summary, plain-language explanation, translate to
  Vietnamese. Each can have its own hotkey.
- **Spoken instructions:** highlight text, press `⌘⇧.`, say what you want ("summarize
  this and read it to me"), press again — the AI follows the instruction and reads the
  result back, no need to open Settings first. Uses the API key you already saved.
- Four text-to-speech providers: Soniox (default, cheapest), Gemini, OpenAI, and
  ElevenLabs (best voice quality, needs its own key). Switch anytime from the "Read
  aloud" tab, with a "Try it" preview button.
- A small floating control bar appears while reading — pause, stop, drag it anywhere,
  it remembers where you left it.
- A new step at the end of first-run setup introduces the feature — skippable with one
  click.

### Fixed

- **Windows: the app hung on "Loading" forever.** The main window opened black with
  nothing but "Loading" on it and never got any further, even though the hotkeys still
  worked and the microphone was still heard. Tauri creates the window *before* the
  backend has finished standing up its settings store, so the page's very first question
  — "what are my settings?" — could be answered with an error, and the UI swallowed that
  error and waited forever for a reply that was never coming. Whether you lost the race
  was luck; WebView2 Runtime 152 starts the page sooner, so machines that updated to it
  lost it every time. The windows are now created only once the backend is ready, and if
  the call fails for some other reason the app retries and then shows the actual error
  with a "Try again" button instead of sitting there.
- **No longer destroys an image sitting on your clipboard.** Before grabbing the
  selection the app clears the clipboard, so a copy that never lands can't be mistaken
  for one that did. If you had just copied an image or a file, that was wiped out with
  no way to get it back. The clear now happens only when the clipboard holds text.

### Security

- Highlighted text is sent to the AI and text-to-speech providers selected in the
  "Read aloud" tab — nowhere else, and no read-aloud history is kept.

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
