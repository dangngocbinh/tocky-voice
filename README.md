# Tocky Voice · Tốc Ký

**Tiếng Việt** · [English](README.en.md)

### Bớt gõ đi, tâm sự bằng giọng nói của bạn.

Giữ một phím ở bất kỳ đâu trên máy, nói, và chữ hiện ra ngay trong ứng dụng bạn đang gõ —
được nhận dạng bởi dịch vụ speech-to-text thời gian thực, và nếu muốn thì được AI viết lại
cho gọn gàng trước khi dán.

Miễn phí và mã nguồn mở. Bạn dùng API key của chính mình, nên không có phí thuê bao và
không có bên trung gian: âm thanh đi thẳng từ máy bạn tới nhà cung cấp bạn chọn.

Viết bằng Tauri v2 (lõi Rust + giao diện React). Giao diện có **tiếng Việt và tiếng Anh**.

<p align="center">
  <img src="brand/screenshots/overlay.png" width="640" alt="Overlay hiện sóng âm và chữ đang được nhận dạng">
</p>

<p align="center"><em>Overlay nổi lên khi bạn nói. Chữ trắng là đã chốt, chữ xám là đang đoán.</em></p>


---

## Cài đặt

### Cách 1 — Tải về chạy luôn (không cần biết lập trình)

Vào [**trang Releases**](../../releases/latest) và tải file hợp với máy bạn:

| Máy | File | Rồi làm gì |
| --- | --- | --- |
| macOS chip Apple (M1–M4) | `..._aarch64.dmg` | Mở file .dmg, kéo app vào Applications |
| macOS chip Intel | `..._x64.dmg` | Tương tự |
| Windows 10/11 | `..._x64-setup.exe` hoặc `..._x64_en-US.msi` | Chạy file |
| Linux | `..._amd64.AppImage` | `chmod +x` rồi chạy |
| Linux (Debian/Ubuntu) | `..._amd64.deb` | `sudo dpkg -i <file>.deb` |

**App chưa mua chứng chỉ nhà phát triển**, nên lần đầu mở máy nào cũng cảnh báo. Phần mềm
mã nguồn mở đều vậy, và đây là cách vượt qua:

- **macOS** — chuột phải vào app → **Open** → **Open**. (Nháy đúp sẽ bị từ chối thẳng.)
- **Windows** — SmartScreen hiện hộp xanh → **More info** → **Run anyway**.

### Cách 2 — Nhờ AI agent cài giúp

Dán đoạn này vào Claude Code, Cursor, Codex, hoặc bất kỳ agent nào chạy được lệnh:

```
Clone https://github.com/dangngocbinh/tocky-voice và build cho máy tôi.

  git clone https://github.com/dangngocbinh/tocky-voice
  cd tocky-voice
  pnpm install
  pnpm tauri build

Cần có trước: Node 20+, pnpm, và Rust toolchain (https://rustup.rs).
Trên Ubuntu/Debian cài thêm:
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
                   libasound2-dev patchelf build-essential libxdo-dev libssl-dev

File cài đặt nằm ở src-tauri/target/release/bundle/. Cài xong mở app lên và làm theo
4 bước hướng dẫn hiện ra lần đầu.
```

### Cách 3 — Tự build

```sh
pnpm install
pnpm tauri build          # file cài đặt nằm ở src-tauri/target/release/bundle/
```

---

## Lần chạy đầu tiên

App dẫn bạn qua 4 bước, và có thể chạy lại bất cứ lúc nào từ
**Cài đặt → Chạy lại hướng dẫn**:

1. **Ngôn ngữ** — tiếng Việt hoặc tiếng Anh (hoặc theo hệ thống).
2. **Dịch vụ giọng nói** — chọn một nhà cung cấp và dán API key miễn phí. Không có key thì
   app không làm được gì cả, nên bước này không cho bỏ qua.
3. **Quyền** (chỉ macOS) — xem bên dưới.
4. **Thử một lần** — nói một câu, xem chữ hiện ra, rồi bắt đầu dùng thật.

### Lấy key giọng nói (miễn phí)

| Nhà cung cấp | Credit miễn phí | Giá streaming | Nên chọn khi |
| --- | --- | --- | --- |
| **Deepgram** — *bắt đầu ở đây* | **$200, không cần thẻ** (~690 giờ) | ~$0.29/giờ | Bạn muốn dùng được ngay mà chưa tốn đồng nào |
| **Soniox** — *chuẩn nhất cho tiếng Việt* | Không có | **$0.12/giờ — rẻ nhất** | Bạn nói tiếng Việt lẫn tiếng Anh trong cùng một câu |
| AssemblyAI | $50, không cần thẻ | $0.15/giờ | Bạn chủ yếu nói tiếng Anh (streaming chưa có tiếng Việt) |

Soniox nghe chuẩn nhất nhưng tính tiền ngay từ phút đầu — bù lại nó rẻ nhất theo giờ.
Deepgram cho $200 miễn phí nên là chỗ bắt đầu hợp lý; khi nào thấy nó nghe sai câu trộn
Việt–Anh thì chuyển sang Soniox. Đổi nhà cung cấp chỉ mất vài giây trong Cài đặt.

Link đăng ký nằm sẵn trong app, ngay cạnh ô nhập key.

### macOS: quyền Accessibility

macOS chặn ứng dụng này gõ chữ vào ứng dụng khác trừ khi bạn cho phép. Thiếu quyền này,
chữ chỉ vào clipboard chứ không bao giờ vào được ứng dụng bạn đang viết.

**System Settings → Privacy & Security → Accessibility** → bật *Tocky Voice*.

> **Đã bật rồi mà app vẫn báo thiếu?** macOS gắn quyền với đúng bản app lúc được cấp, nên
> app cập nhật hoặc build lại là phải cấp lại — trong khi công tắc cũ vẫn hiện **bật**,
> rất dễ hiểu nhầm. Chọn dòng đó, bấm **−**, rồi bấm **+** và thêm app vào lại.
> Tắt/bật công tắc không có tác dụng.

---

## Dùng như thế nào

Bấm phím tắt trong ứng dụng bất kỳ, nói, bấm lại lần nữa. Chữ được dán ngay chỗ con trỏ.

| Việc | Mặc định |
| --- | --- |
| Bắt đầu / dừng | `Control+Alt+D` |
| Giữ để nói | Right Option *(chỉ macOS — Windows/Linux dùng tổ hợp phím)* |
| Huỷ lần ghi này | `Control+Alt+X` |
| Chuyển chế độ | `Control+Alt+M` |

Đổi lại được hết trong **Cài đặt → Phím tắt**.

### Chế độ (Modes)

Một chế độ là một câu lệnh cho AI cộng với cách giao chữ, nên cùng một câu nói có thể ra
những thứ khác nhau. Có sẵn 4 chế độ, và bạn tự thêm được, mỗi cái một phím tắt riêng:

- **Raw** — không qua AI, dán thẳng bản nhận dạng (nhanh nhất, gần như không tốn thêm thời gian)
- **Clean** — sửa chính tả, dấu câu, bỏ từ đệm; giữ nguyên thuật ngữ tiếng Anh
- **Prompt** — viết lại thành một yêu cầu mạch lạc cho AI coding agent
- **Email** — viết lại thành email/tin nhắn công việc trang trọng

### AI viết lại (tuỳ chọn)

Anthropic Claude, OpenAI, Google Gemini, DeepSeek, Qwen, Moonshot Kimi, Zhipu GLM,
MiniMax, Groq, xAI, OpenRouter, Ollama chạy máy nhà, hoặc bất kỳ endpoint nào tương thích
OpenAI. Danh sách model được lấy trực tiếp từ nhà cung cấp bạn chọn, nên model ra hôm nay
là hôm nay chọn được.

Đo trên chế độ *Clean*, câu tiếng Việt có xen thuật ngữ tiếng Anh:

| Nhà cung cấp / model | Thời gian |
| --- | --- |
| DeepSeek `deepseek-v4-flash` | ~1.1 giây |
| OpenAI `gpt-4.1-mini` | ~1.7 giây |

Nếu thấy chậm, kiểm tra xem model có phải loại *reasoning* không: loại đó đốt vài giây để
"suy nghĩ ẩn" — thứ không ai đọc — trước khi trả lời. DeepSeek V4 từ 10.8 giây xuống còn
1.9 giây sau khi tắt phần đó đi, và app giờ tự tắt giúp bạn.

---

## Giao diện

| | |
| --- | --- |
| <img src="brand/screenshots/providers.png" alt="Chọn nhà cung cấp giọng nói"> | <img src="brand/screenshots/modes.png" alt="Chỉnh chế độ"> |
| **Nhà cung cấp** — badge cho biết cái nào chuẩn tiếng Việt, cái nào miễn phí | **Chế độ** — mỗi chế độ một câu lệnh cho AI và một phím tắt riêng |
| <img src="brand/screenshots/about.png" alt="Tab giới thiệu"> | |
| **Giới thiệu** — link đăng ký thẳng tới từng nhà cung cấp | |

---

## Hỗ trợ nền tảng

App được viết và dùng hằng ngày trên macOS. CI biên dịch và chạy test cả ba nền tảng ở mỗi
lần push, nhưng **Windows và Linux chưa được dùng thử bằng tay** — hãy coi là bản beta và
báo lại giúp nếu có gì hỏng.

| | macOS | Windows | Linux |
| --- | --- | --- | --- |
| Nói → dán chữ | ✅ | ✅ biên dịch được, chưa thử tay | ⚠️ xem ghi chú dưới |
| Phím tắt toàn cục | ✅ | ✅ biên dịch được, chưa thử tay | ⚠️ xem ghi chú dưới |
| Giữ một phím đơn để nói | ✅ | ❌ dùng tổ hợp phím | ❌ dùng tổ hợp phím |
| Trả focus đúng ứng dụng lúc bắt đầu ghi | ✅ | chưa làm — ẩn overlay đi thì focus tự về, thực tế vẫn đúng | như Windows |
| Quyền file chứa key | `0600` | theo ACL thư mục AppData của tài khoản | `0600` |

**Lưu ý cho Linux:** Wayland siết chặt việc giả lập phím và bắt phím tắt toàn cục hơn X11
nhiều. Nếu phím tắt hoặc thao tác dán không ăn trên phiên Wayland, cách khắc phục là dùng
phiên X11. Cả hai đều chưa được kiểm chứng.

---

## API key của bạn được cất ở đâu

Mặc định nằm trong `credentials.json` — quyền `0600`, trong thư mục dữ liệu `0700`, nên
không tài khoản nào khác trên máy đọc được. Không bao giờ nằm trong `settings.json`, và
giao diện chỉ hỏi được là *có key hay chưa*; không có lệnh nào đọc key ra cả.

**Cài đặt → Nhà cung cấp** cho phép chuyển sang keychain của hệ điều hành (Keychain /
Credential Manager / Secret Service). Cách đó an toàn hơn vì hệ điều hành cấp quyền cho
đúng file binary này. Hãy bật **sau khi app đã được ký số**. Với bản macOS chưa ký, keychain
sẽ hỏi **mật khẩu đăng nhập máy** mỗi lần đọc — một công cụ cứ hỏi mật khẩu tài khoản của
bạn là đang tập cho bạn thói quen bị lừa đảo, nên đó không phải mặc định. Chuyển qua lại
đều tự động chuyển key theo.

## Dữ liệu lưu trên máy

macOS: `~/Library/Application Support/pro.mecode.tockyvoice/`
Windows: `%APPDATA%\pro.mecode.tockyvoice\`
Linux: `~/.config/pro.mecode.tockyvoice/`

- `settings.json` — mọi thứ trừ API key
- `credentials.json` — API key, chỉ chủ máy đọc được
- `history.json` — bản gốc và bản đã chỉnh của từng lần nói
- `recordings/` — file WAV từng lần ghi, tự xoá theo số ngày bạn đặt

Không có gì được gửi đi đâu ngoài dịch vụ giọng nói và AI mà bạn tự cấu hình. Không có
telemetry.

---

## Cách hoạt động

```
phím tắt ─▶ thu mic (cpal, 16 kHz mono) ─▶ WebSocket tới dịch vụ STT ─▶ bản nhận dạng
                    │                                                        │
                    └─▶ overlay nổi: sóng âm + chữ hiện dần                   ▼
                                                                AI viết lại (theo chế độ)
                                                                              │
                                                                              ▼
                                                        clipboard + ⌘V vào app đang focus
```

Overlay không bao giờ chiếm focus bàn phím. Đó chính là mấu chốt: ứng dụng bạn đang gõ vẫn
ở phía trước, nên phím dán giả lập rơi vào đó chứ không rơi vào app này.

## Phát triển

```sh
pnpm install
pnpm tauri dev                             # chạy với hot reload
pnpm tauri build --debug --bundles app     # ra file .app cấp quyền được
```

Quyền hệ thống gắn với đường dẫn và chữ ký của binary, nên mọi thứ liên quan tới micro hay
thao tác dán phải chạy qua file .app đã đóng gói — chạy `cargo run` trần sẽ bị từ chối.

> **Chạy app từ terminal trên macOS cho kết quả sai lệch**: tiến trình con của terminal
> thừa hưởng quyền Accessibility của terminal, nên nó báo là đã có quyền trong khi cùng
> bản app đó mở từ Dock thì không. Luôn kiểm chứng bằng cách mở từ Dock.

## Chạy test

```sh
cd src-tauri
cargo test --lib                           # test đơn vị, không cần mạng
```

Các test round-trip gọi API thật nên bị đánh dấu `#[ignore]`. `--test-threads=1` là bắt
buộc: gói miễn phí giới hạn số phiên streaming đồng thời, chạy song song sẽ fail vì lý do
chẳng liên quan gì tới code.

```sh
cd src-tauri
FVT_TEST_WAV=/đường/dẫn/16k-mono.wav \
SONIOX_API_KEY=... DEEPGRAM_API_KEY=... ASSEMBLYAI_API_KEY=... \
cargo test --test stt_provider_round_trip -- --ignored --nocapture --test-threads=1

FVT_LLM_PRESET=deepseek FVT_LLM_MODEL=deepseek-v4-flash FVT_LLM_KEY=sk-... \
FVT_SETTINGS="$HOME/Library/Application Support/pro.mecode.tockyvoice/settings.json" \
cargo test --test mode_round_trip -- --ignored --nocapture --test-threads=1
```

Tạo file âm thanh tiếng Việt để test trên macOS:

```sh
say -v Linh -o sample.aiff "Xin chào, hôm nay tôi sẽ deploy cái API này lên server."
afconvert -f WAVE -d LEI16@16000 -c 1 sample.aiff sample-vi.wav
```

## Phát hành bản mới

Đẩy một tag lên là CI tự build cả bốn cấu hình và tạo draft release:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

---

## Ủng hộ tác giả

Tocky Voice miễn phí và sẽ luôn miễn phí. Nếu nó giúp ích cho bạn, bạn có thể ủng hộ để
tác giả tiếp tục làm những sản phẩm tiếp theo:

| | |
| --- | --- |
| **MB Bank** | `0899279774` — Đặng Ngọc Bình |
| **Vietcombank** | `0251002681598` — Đặng Ngọc Bình |

Hoặc quét QR và xem thêm cách khác tại **[dangngocbinh.com/lien-he](https://dangngocbinh.com/lien-he/)**.

🎓 Tác giả cũng dạy về AI Automation, VibeCode, AI Agent và tự động hoá —
**[xem các khoá học tại ME Code](https://mecode.pro/khoa-hoc)**.

Không ủng hộ cũng không sao — cứ dùng thoải mái, và kể cho người khác nghe là đủ vui rồi.

---

## Giấy phép

[MIT](LICENSE) — dùng, sửa, phát hành, bán lại đều được. Điều kiện duy nhất là giữ lại dòng
bản quyền trong mọi bản sao hoặc phần đáng kể của mã nguồn, để công trình vẫn chỉ về nơi
nó bắt đầu.

Fork và dự án của học viên đều được hoan nghênh, không cần xin phép.

### Về cái tên và logo

Giấy phép MIT bao phủ **mã nguồn**. Nó không cấp quyền với cái tên *Tocky Voice* hay
*Tốc Ký*, tên *ME Code*, hay logo của chúng — những thứ đó thuộc về nhãn hiệu chứ không
thuộc giấy phép bản quyền, và không nằm trong phần được cấp ở trên.

Nói dễ hiểu: xây gì trên nền này cũng được, nhưng hãy đặt tên riêng và icon riêng cho bản
của bạn, đừng phát hành thứ trông như từ chúng tôi ra. Ghi "dựa trên Tocky Voice của ME
Code" thì hoàn toàn ổn và rất được hoan nghênh.

## Tác giả

Được xây bởi [ME Code](https://mecode.pro) — AI Automation Academy & Product Lab.
