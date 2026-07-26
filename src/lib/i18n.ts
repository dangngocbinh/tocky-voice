/**
 * Interface language.
 *
 * `en` is the source of truth and `vi` is typed against it, so a missing translation
 * is a compile error rather than a string that silently falls back to English.
 *
 * Components read a whole typed dictionary (`t.dictate.title`) instead of calling a
 * lookup with a string path — no runtime key errors, and renaming a key is a refactor
 * the compiler can follow.
 */

import { createContext, useContext } from "react";

export type UiLanguage = "system" | "en" | "vi";

/** Resolve `system` against the OS locale the webview reports. */
export function resolveLanguage(preference: UiLanguage): "en" | "vi" {
  if (preference !== "system") return preference;
  const locales = navigator.languages?.length
    ? navigator.languages
    : [navigator.language ?? "en"];
  // Anything not clearly Vietnamese falls back to English, per the brief.
  return locales.some((l) => l.toLowerCase().startsWith("vi")) ? "vi" : "en";
}

const en = {
  nav: {
    dictate: "Dictate",
    modes: "Modes",
    providers: "Providers",
    // The tab holds language, sounds, history and startup as well as hotkeys —
    // naming it after just one of those hides the rest.
    hotkeys: "Settings",
    history: "History",
    about: "About",
    idle: "idle",
    loading: "Loading",
  },

  phase: {
    idle: "Ready",
    recording: "Listening",
    transcribing: "Transcribing",
    refining: "Polishing",
    pasting: "Pasting",
  },

  common: {
    copy: "Copy",
    copied: "Copied",
    delete: "Delete",
    cancel: "Cancel",
    close: "Close",
    save: "Save",
    remove: "Remove",
    clear: "Clear",
    list: "List",
    other: "Other — type an id…",
    fetch: "Fetch",
    systemDefault: "System default",
    openSettings: "Open Settings",
  },

  dictate: {
    title: "Dictate",
    lede:
      "Press your hotkey in any app and talk. The text is transcribed, optionally rewritten by the active mode, then pasted where your cursor was.",
    accessibilityTitle: "Accessibility permission needed.",
    accessibilityBody:
      "Without it macOS blocks the paste keystroke and the hold-to-talk key, so text reaches your clipboard but never the app you are typing in.",
    start: "Start dictation",
    stop: "Stop & paste",
    working: "Working…",
    copyText: "Copy text",
    empty: "Nothing yet. Start talking and the words appear here as they are recognised.",
    inputSection: "Input",
    microphone: "Microphone",
    microphoneHint: "Switch inputs without leaving this screen.",
    mode: "Mode",
    modeHint: "Decides what the AI does with the transcript before it is pasted.",
    raw: "raw",
  },

  modes: {
    title: "Modes",
    lede:
      "Each mode is a set of instructions for the AI pass. Give one a hotkey and it switches mode and starts recording in a single press.",
    add: "+ Add mode",
    active: "active",
    bound: "bound",
    name: "Name",
    hotkey: "Hotkey",
    hotkeyHint: "Switches to this mode and starts recording.",
    notSet: "Not set",
    aiCleanup: "Run AI cleanup",
    aiCleanupHint: "Off is the fastest path — the raw transcript is pasted immediately.",
    output: "After processing",
    outputPaste: "Paste into the focused app",
    outputCopy: "Copy to clipboard only",
    prompt: "Instructions",
    promptHint: "Sent as the system prompt with the raw transcript as the message.",
    deleteMode: "Delete mode",
    lastMode: "The last mode cannot be deleted.",
    newModeName: "New mode",
    newModePrompt: "Fix spelling and punctuation. Return only the corrected text.",
  },

  providers: {
    title: "Providers",
    lede:
      "Speech recognition runs over a streaming connection; the AI pass is a single request afterwards. Both use your own accounts.",
    speechSection: "Speech recognition",
    keySet: "key set",
    noKey: "no key",
    getKey: "Get a key ↗",
    language: "Language",
    languageHint: "Deepgram takes exactly one language.",
    hints: "Language hints",
    hintsHint:
      "Soniox accepts several at once — this is what makes a sentence that mixes Vietnamese and English work. Click to toggle; the number shows priority.",
    aiSection: "AI cleanup",
    provider: "Provider",
    providerHint: "Anything OpenAI-compatible works, including a local Ollama.",
    model: "Model",
    modelHint: "Reasoning models produce better text but add seconds before the paste.",
    baseUrl: "Base URL",
    baseUrlHint: "Must be https, unless it is on localhost.",
    apiKey: "API key",
    apiKeyHint: "Saved to the credential store, never to settings.",
    getKeyFrom: "Get one from",
    keyPlaceholder: "Paste API key",
    keySaved: "Saved — paste a new key to replace",
    testSection: "Connection check",
    test: "Test model",
    testing: "Testing…",
    storageSection: "Credential storage",
    useKeychain: "Use the OS keychain",
    useKeychainHint:
      "Off by default: keys live in a file only your account can read. The keychain is stronger, but an unsigned build makes macOS ask for your login password on every read — turn this on once the app is code-signed. Existing keys move across automatically.",
  },

  behaviour: {
    title: "Settings",
    lede:
      "Language, shortcuts, sounds, history and startup. Hotkeys are global — they work from whichever app you are typing in.",
    generalSection: "General",
    uiLanguage: "Interface language",
    uiLanguageHint: "Follows your system language unless you pick one.",
    followSystem: "Follow system",
    shortcutsSection: "Shortcuts",
    toggle: "Start / stop",
    toggleHint: "Press once to record, again to paste.",
    cancel: "Cancel",
    cancelHint: "Throws the current take away.",
    nextMode: "Next mode",
    pttSection: "Push to talk",
    pttTrigger: "Trigger",
    pttTriggerHint: "Hold to record, release to transcribe and paste.",
    pttModifier: "Hold one modifier key",
    pttShortcut: "Hold a combination",
    pttDisabled: "Disabled",
    pttKey: "Key",
    pttKeyHint:
      "Watched by a listen-only event tap, so the key keeps working normally everywhere else. macOS only.",
    pttCombo: "Combination",
    feedbackSection: "Feedback",
    sounds: "Sound cues",
    soundsHint:
      "Short tones on start, stop, done and cancel, so you can dictate without watching the screen.",
    volume: "Volume",
    historySection: "History",
    keepHistory: "Keep a history",
    keepHistoryHint: "Stored in a file only your account can read.",
    keepAudio: "Save recordings",
    keepAudioHint: "Replay the audio when a transcript comes out wrong.",
    maxEntries: "Entries to keep",
    retention: "Delete recordings after",
    retentionHint: "Days. The text entry is kept.",
    startupSection: "Startup",
    autostart: "Launch at login",
  },

  history: {
    title: "History",
    lede:
      "Everything you have dictated, newest first — so a lost or mangled paste can be recovered instead of re-recorded.",
    empty: "Nothing here yet.",
    emptyHint: "Dictations you make will be saved so you can copy them again later.",
    entry: "entry",
    entries: "entries",
    readFull: "Read full text",
    chars: "chars",
    rawSummary: "Raw transcript, before AI cleanup",
    copyRaw: "Copy raw",
    showRaw: "Show raw transcript",
    showPolished: "Show polished",
    clearAll: "Clear all history",
    clearHint: "Deleting clears the saved recordings too.",
    confirmClear: "Delete all history and saved recordings?",
  },

  about: {
    title: "About",
    lede:
      "Tocky Voice is open source. It uses your own provider accounts, so nothing is billed through anyone else and no audio passes through a middleman.",
    madeBy: "Made by",
    sttSection: "Speech recognition — pick one",
    sttHint: "Required. This is what turns your voice into text.",
    llmSection: "AI cleanup — optional",
    llmHint:
      "Only needed for modes that rewrite the transcript. The Raw mode works without any of these.",
    signUp: "Sign up ↗",
    getKey: "Get a key ↗",
    beforeSection: "Before you hand this to someone",
    privacy:
      "Your voice is streamed to the speech provider, and the transcript is then sent to the AI provider — both under the account whose key you entered. Anything you dictate reaches those two companies. Keys are stored in a file only your user account can read, or in the OS keychain if you switch that on in Providers. Credit amounts above are what the vendors advertise and can change without notice — check the signup page.",
  },

  overlay: {
    speak: "Start speaking…",
    release: "release",
    stop: "stop",
  },

  recorder: {
    press: "Press keys…",
    clickToSet: "Click to set",
    hint: "esc cancel · ⌫ clear",
    needModifier: "Add a modifier (⌃ ⌥ ⇧ ⌘), or use a function key",
  },
  onboarding: {
    next: "Continue",
    back: "Back",
    skip: "Skip setup",
    finish: "Start using it",
    recommended: "recommended",
    recheck: "Check again",
    checking: "Checking…",

    tagline: "Don't type. Just talk.",
    languageTitle: "Welcome to Tocky Voice",
    languageBody:
      "Hold a key anywhere on your Mac, talk, and what you said is typed into whatever app you are in. Four short steps and you are ready. First, which language should this app be in?",

    speechTitle: "Connect a speech service",
    speechBody:
      "Your voice is transcribed by an online service, so you need a free key from one of these. Pick one, get the key, paste it below.",
    getFreeKey: "Get a free key",
    keyStep1: "Open {provider} and sign up — no card needed for the free credit.",
    keyStep2: "Copy the key it gives you and paste it here.",
    keySaved: "{provider} key saved.",

    permissionTitle: "Let macOS type for you",
    permissionBody:
      "macOS blocks apps from typing into other apps unless you allow it. Without this, your words reach the clipboard but never land in the app you are writing in.",
    permissionMissing: "Not granted yet",
    permissionGranted: "Granted — you are all set",
    permissionStep1: "Click Open Settings below.",
    permissionStep2: "Find Tocky Voice and switch it on.",
    permissionStep3:
      "Already switched on but still red? macOS ties the permission to the exact copy of the app it was granted to, so a rebuilt or updated app needs it again: select the row, click −, then + and add the app back.",
    permissionStep4: "Come back here — this turns green on its own.",

    tryTitle: "Try it once",
    tryBody: "Press the button, say a sentence, and press it again. Check the words look right.",
    tryHint: "Your words will appear here…",
    tryStart: "Start talking",
    keyToggle: "start and stop, from any app",
    keyHold: "or hold this key and talk",
    tryFinale:
      "That is the whole app. Now click into any other app — a browser, Notes, a chat box — press the hotkey, and talk. The text is pasted where your cursor is.",

    rerunTitle: "Setup walkthrough",
    rerunBody: "Run the four-step setup again — handy if something stopped working.",
    rerun: "Run setup again",
  },

  errors: {
    needs_accessibility:
      "Copied to the clipboard, but pasting needs Accessibility permission. Grant it to Tocky Voice in System Settings → Privacy & Security → Accessibility.",
    no_paste_target:
      "Copied to the clipboard. There was nothing to paste into — this take started while Tocky Voice was the front window. Click into the app you want the text in first, then press your hotkey.",
    delivery_failed:
      "Could not deliver the text. Your words are safe — open the History tab to copy them.",
    no_stt_key: "No speech API key saved. Add one in Providers before recording.",
    no_llm_key: "No AI provider key saved. Pasting the raw transcript instead.",
    cleanup_failed: "AI cleanup failed. Pasting the raw transcript instead.",
    mic_unavailable: "Could not open the microphone.",
    transcription_failed: "Transcription failed.",
  },

  stt: {
    best_vietnamese: "Best for Vietnamese",
    free_credit: "Free {amount}",
    paid: "Paid — ${price}/hour",
    soniox:
      "The most accurate when you mix Vietnamese and English in one sentence. Billed from the first minute, but the cheapest per hour.",
    deepgram:
      "The largest free credit — enough for hundreds of hours. Handles Vietnamese, but noticeably weaker than Soniox on sentences that mix in English.",
    assembly_ai:
      "Very accurate on English, but its streaming tier has no Vietnamese yet. Billed on how long the connection stays open, not on how long you speak.",
  },
} as const;

type Dictionary = {
  -readonly [S in keyof typeof en]: { -readonly [K in keyof (typeof en)[S]]: string };
};

const vi: Dictionary = {
  nav: {
    dictate: "Đọc",
    modes: "Chế độ",
    providers: "Nhà cung cấp",
    hotkeys: "Cài đặt",
    history: "Lịch sử",
    about: "Giới thiệu",
    idle: "chờ",
    loading: "Đang tải",
  },

  phase: {
    idle: "Sẵn sàng",
    recording: "Đang nghe",
    transcribing: "Đang nhận dạng",
    refining: "Đang tinh chỉnh",
    pasting: "Đang dán",
  },

  common: {
    copy: "Chép",
    copied: "Đã chép",
    delete: "Xoá",
    cancel: "Huỷ",
    close: "Đóng",
    save: "Lưu",
    remove: "Gỡ",
    clear: "Xoá",
    list: "Danh sách",
    other: "Khác — tự nhập id…",
    fetch: "Tải về",
    systemDefault: "Mặc định hệ thống",
    openSettings: "Mở System Settings",
  },

  dictate: {
    title: "Đọc",
    lede:
      "Bấm phím tắt ở bất kỳ ứng dụng nào rồi nói. Lời nói được chuyển thành chữ, có thể được AI viết lại theo chế độ đang chọn, rồi dán vào đúng chỗ con trỏ.",
    accessibilityTitle: "Cần quyền Accessibility.",
    accessibilityBody:
      "Thiếu quyền này macOS sẽ chặn thao tác dán và phím giữ-để-nói, nên chữ chỉ vào clipboard chứ không vào được ứng dụng anh đang gõ.",
    start: "Bắt đầu đọc",
    stop: "Dừng & dán",
    working: "Đang xử lý…",
    copyText: "Chép đoạn này",
    empty: "Chưa có gì. Cứ nói, chữ sẽ hiện ở đây ngay khi nhận dạng được.",
    inputSection: "Đầu vào",
    microphone: "Micro",
    microphoneHint: "Đổi micro ngay tại đây, không cần vào Cài đặt.",
    mode: "Chế độ",
    modeHint: "Quyết định AI làm gì với văn bản trước khi dán.",
    raw: "thô",
  },

  modes: {
    title: "Chế độ",
    lede:
      "Mỗi chế độ là một bộ hướng dẫn cho AI. Gán phím tắt cho chế độ nào thì một lần bấm vừa chuyển chế độ vừa bắt đầu thu.",
    add: "+ Thêm chế độ",
    active: "đang dùng",
    bound: "có phím",
    name: "Tên",
    hotkey: "Phím tắt",
    hotkeyHint: "Chuyển sang chế độ này và bắt đầu thu.",
    notSet: "Chưa đặt",
    aiCleanup: "Cho AI tinh chỉnh",
    aiCleanupHint: "Tắt là nhanh nhất — văn bản thô được dán ngay.",
    output: "Sau khi xử lý",
    outputPaste: "Dán vào ứng dụng đang mở",
    outputCopy: "Chỉ chép vào clipboard",
    prompt: "Hướng dẫn cho AI",
    promptHint: "Gửi làm system prompt, kèm văn bản thô làm nội dung.",
    deleteMode: "Xoá chế độ",
    lastMode: "Không xoá được chế độ cuối cùng.",
    newModeName: "Chế độ mới",
    newModePrompt: "Sửa chính tả và dấu câu. Chỉ trả về văn bản đã sửa.",
  },

  providers: {
    title: "Nhà cung cấp",
    lede:
      "Nhận dạng giọng nói chạy qua kết nối streaming; phần AI tinh chỉnh là một lượt gọi sau đó. Cả hai đều dùng tài khoản của anh.",
    speechSection: "Nhận dạng giọng nói",
    keySet: "đã có key",
    noKey: "chưa có key",
    getKey: "Lấy key ↗",
    language: "Ngôn ngữ",
    languageHint: "Deepgram chỉ nhận đúng một ngôn ngữ.",
    hints: "Gợi ý ngôn ngữ",
    hintsHint:
      "Soniox nhận nhiều ngôn ngữ cùng lúc — đây là thứ giúp câu pha tiếng Việt lẫn tiếng Anh chạy đúng. Bấm để bật/tắt; số là thứ tự ưu tiên.",
    aiSection: "AI tinh chỉnh",
    provider: "Nhà cung cấp",
    providerHint: "Bất kỳ endpoint nào tương thích OpenAI đều dùng được, kể cả Ollama chạy máy nhà.",
    model: "Model",
    modelHint: "Model biết suy luận cho chữ đẹp hơn nhưng chậm thêm vài giây trước khi dán.",
    baseUrl: "Base URL",
    baseUrlHint: "Bắt buộc https, trừ khi chạy ở localhost.",
    apiKey: "API key",
    apiKeyHint: "Lưu vào kho khoá, không bao giờ ghi vào file cài đặt.",
    getKeyFrom: "Lấy key ở",
    keyPlaceholder: "Dán API key",
    keySaved: "Đã lưu — dán key mới để thay",
    testSection: "Kiểm tra kết nối",
    test: "Thử model",
    testing: "Đang thử…",
    storageSection: "Nơi lưu khoá",
    useKeychain: "Dùng keychain hệ thống",
    useKeychainHint:
      "Mặc định tắt: key nằm trong file chỉ tài khoản của anh đọc được. Keychain an toàn hơn, nhưng bản chưa ký sẽ khiến macOS hỏi mật khẩu đăng nhập mỗi lần đọc — chỉ nên bật khi app đã được ký. Key hiện có sẽ tự chuyển sang.",
  },

  behaviour: {
    title: "Cài đặt",
    lede:
      "Ngôn ngữ, phím tắt, âm thanh, lịch sử và khởi động cùng máy. Phím tắt là toàn cục — dùng được từ bất kỳ ứng dụng nào.",
    generalSection: "Chung",
    uiLanguage: "Ngôn ngữ giao diện",
    uiLanguageHint: "Theo ngôn ngữ hệ thống, trừ khi anh tự chọn.",
    followSystem: "Theo hệ thống",
    shortcutsSection: "Phím tắt",
    toggle: "Bắt đầu / dừng",
    toggleHint: "Bấm lần đầu để thu, bấm lần nữa để dán.",
    cancel: "Huỷ",
    cancelHint: "Bỏ luôn lần thu đang chạy.",
    nextMode: "Chế độ kế tiếp",
    pttSection: "Giữ để nói",
    pttTrigger: "Kiểu kích hoạt",
    pttTriggerHint: "Giữ để thu, thả ra là nhận dạng rồi dán.",
    pttModifier: "Giữ một phím bổ trợ",
    pttShortcut: "Giữ một tổ hợp",
    pttDisabled: "Tắt",
    pttKey: "Phím",
    pttKeyHint:
      "Theo dõi bằng event tap chỉ-nghe, nên phím đó vẫn hoạt động bình thường ở mọi nơi khác. Chỉ có trên macOS.",
    pttCombo: "Tổ hợp",
    feedbackSection: "Phản hồi",
    sounds: "Âm báo",
    soundsHint:
      "Tiếng bíp ngắn khi bắt đầu, dừng, xong và huỷ — để anh đọc mà không cần nhìn màn hình.",
    volume: "Âm lượng",
    historySection: "Lịch sử",
    keepHistory: "Lưu lịch sử",
    keepHistoryHint: "Lưu trong file chỉ tài khoản của anh đọc được.",
    keepAudio: "Lưu file ghi âm",
    keepAudioHint: "Nghe lại được khi nhận dạng ra sai.",
    maxEntries: "Số bản ghi giữ lại",
    retention: "Xoá file ghi âm sau",
    retentionHint: "Số ngày. Phần chữ vẫn được giữ.",
    startupSection: "Khởi động",
    autostart: "Chạy khi đăng nhập",
  },

  history: {
    title: "Lịch sử",
    lede:
      "Mọi thứ anh đã đọc, mới nhất trước — để lỡ dán hụt hay dán sai thì lấy lại được thay vì phải đọc lại.",
    empty: "Chưa có gì ở đây.",
    emptyHint: "Những lần đọc của anh sẽ được lưu lại để chép lại sau.",
    entry: "bản ghi",
    entries: "bản ghi",
    readFull: "Đọc toàn văn",
    chars: "ký tự",
    rawSummary: "Văn bản thô, trước khi AI sửa",
    copyRaw: "Chép bản thô",
    showRaw: "Xem bản thô",
    showPolished: "Xem bản đã sửa",
    clearAll: "Xoá toàn bộ lịch sử",
    clearHint: "Xoá thì mất luôn các file ghi âm đã lưu.",
    confirmClear: "Xoá toàn bộ lịch sử và file ghi âm?",
  },

  about: {
    title: "Giới thiệu",
    lede:
      "Tocky Voice là mã nguồn mở. App dùng tài khoản nhà cung cấp của chính anh, nên không ai đứng giữa thu tiền và không có âm thanh nào đi vòng qua bên thứ ba.",
    madeBy: "Thực hiện bởi",
    sttSection: "Nhận dạng giọng nói — chọn một",
    sttHint: "Bắt buộc. Đây là thứ biến giọng nói thành chữ.",
    llmSection: "AI tinh chỉnh — tuỳ chọn",
    llmHint:
      "Chỉ cần khi dùng chế độ có viết lại văn bản. Chế độ Raw chạy được mà không cần cái nào ở đây.",
    signUp: "Đăng ký ↗",
    getKey: "Lấy key ↗",
    beforeSection: "Trước khi đưa app này cho người khác",
    privacy:
      "Giọng nói của anh được truyền tới nhà cung cấp nhận dạng, rồi văn bản được gửi tiếp tới nhà cung cấp AI — cả hai đều chạy trên tài khoản mà anh nhập key. Mọi thứ anh đọc đều tới tay hai công ty đó. Key lưu trong file chỉ tài khoản của anh đọc được, hoặc trong keychain hệ thống nếu anh bật ở tab Nhà cung cấp. Các con số credit ở trên là do hãng quảng cáo và có thể đổi bất cứ lúc nào — hãy kiểm tra lại ở trang đăng ký.",
  },

  overlay: {
    speak: "Nói đi…",
    release: "thả để xong",
    stop: "dừng",
  },

  recorder: {
    press: "Gõ phím…",
    clickToSet: "Bấm để đặt",
    hint: "esc huỷ · ⌫ xoá",
    needModifier: "Cần thêm phím bổ trợ (⌃ ⌥ ⇧ ⌘), hoặc dùng phím F",
  },
  onboarding: {
    next: "Tiếp tục",
    back: "Quay lại",
    skip: "Bỏ qua",
    finish: "Bắt đầu dùng",
    recommended: "nên chọn",
    recheck: "Kiểm tra lại",
    checking: "Đang kiểm tra…",

    tagline: "Đã đến lúc bớt gõ lại. Vì Tocky sẽ ghi ra chính xác những gì bạn nói trên máy tính.",
    languageTitle: "Chào mừng đến Tocky Voice",
    languageBody:
      "Giữ một phím ở bất kỳ đâu trên máy, nói, và chữ được gõ thẳng vào ứng dụng bạn đang mở. Bốn bước ngắn là xong. Trước tiên, bạn muốn app hiển thị bằng ngôn ngữ nào?",

    speechTitle: "Kết nối dịch vụ nhận dạng giọng nói",
    speechBody:
      "Giọng của bạn được chuyển thành chữ bởi một dịch vụ online, nên bạn cần một key miễn phí. Chọn một nhà cung cấp, lấy key, dán vào ô bên dưới.",
    getFreeKey: "Lấy key miễn phí",
    keyStep1: "Mở {provider} và đăng ký — không cần thẻ để nhận credit miễn phí.",
    keyStep2: "Copy key họ cấp rồi dán vào đây.",
    keySaved: "Đã lưu key {provider}.",

    permissionTitle: "Cho phép macOS gõ giúp bạn",
    permissionBody:
      "macOS chặn ứng dụng gõ chữ vào ứng dụng khác trừ khi bạn cho phép. Thiếu quyền này, chữ chỉ vào clipboard chứ không bao giờ vào ứng dụng bạn đang viết.",
    permissionMissing: "Chưa được cấp",
    permissionGranted: "Đã cấp — xong rồi",
    permissionStep1: "Bấm Mở cài đặt bên dưới.",
    permissionStep2: "Tìm Tocky Voice rồi bật lên.",
    permissionStep3:
      "Đã bật rồi mà vẫn đỏ? macOS gắn quyền với đúng bản app lúc được cấp, nên app build lại hoặc cập nhật thì phải cấp lại: chọn dòng đó, bấm −, rồi bấm + thêm app vào lại.",
    permissionStep4: "Quay lại đây — dòng này sẽ tự chuyển xanh.",

    tryTitle: "Thử một lần",
    tryBody: "Bấm nút, nói một câu, rồi bấm lại. Xem chữ ra có đúng không.",
    tryHint: "Chữ của bạn sẽ hiện ở đây…",
    tryStart: "Bắt đầu nói",
    keyToggle: "bật và tắt, từ bất kỳ ứng dụng nào",
    keyHold: "hoặc giữ phím này rồi nói",
    tryFinale:
      "Vậy là xong. Giờ bấm vào một ứng dụng bất kỳ — trình duyệt, Notes, ô chat — bấm phím tắt rồi nói. Chữ sẽ được dán ngay chỗ con trỏ.",

    rerunTitle: "Hướng dẫn cài đặt",
    rerunBody: "Chạy lại 4 bước cài đặt — hữu ích khi có gì đó ngừng hoạt động.",
    rerun: "Chạy lại hướng dẫn",
  },

  errors: {
    needs_accessibility:
      "Đã copy vào clipboard, nhưng để dán được thì cần quyền Accessibility. Cấp cho Tocky Voice trong System Settings → Privacy & Security → Accessibility.",
    no_paste_target:
      "Đã copy vào clipboard. Không có chỗ nào để dán — lần ghi này bắt đầu lúc cửa sổ Tocky Voice đang ở phía trước. Bấm vào ứng dụng bạn muốn chữ vào đó trước, rồi mới bấm phím tắt.",
    delivery_failed:
      "Không giao được chữ. Lời bạn nói không mất — mở tab Lịch sử để copy lại.",
    no_stt_key: "Chưa lưu API key nhận dạng giọng nói. Vào Nhà cung cấp thêm key trước khi ghi.",
    no_llm_key: "Chưa lưu key cho nhà cung cấp AI. Dán bản gốc chưa qua AI.",
    cleanup_failed: "AI viết lại thất bại. Dán bản gốc chưa qua AI.",
    mic_unavailable: "Không mở được micro.",
    transcription_failed: "Nhận dạng giọng nói thất bại.",
  },

  stt: {
    best_vietnamese: "Chuẩn tiếng Việt",
    free_credit: "Miễn phí {amount}",
    paid: "Trả phí — ${price}/giờ",
    soniox:
      "Nghe chuẩn nhất khi bạn nói tiếng Việt lẫn tiếng Anh trong cùng một câu. Tính tiền ngay từ phút đầu, nhưng lại rẻ nhất theo giờ.",
    deepgram:
      "Credit miễn phí lớn nhất — đủ dùng hàng trăm giờ. Nghe tiếng Việt được, nhưng câu trộn tiếng Anh thì kém Soniox rõ rệt.",
    assembly_ai:
      "Rất chính xác với tiếng Anh, nhưng bản streaming chưa hỗ trợ tiếng Việt. Tính tiền theo thời gian mở kết nối, không phải thời lượng nói.",
  },
};

const DICTIONARIES = { en, vi } as const;

export const TranslationContext = createContext<Dictionary>(en as Dictionary);

export function dictionaryFor(language: "en" | "vi"): Dictionary {
  return DICTIONARIES[language] as Dictionary;
}

/** The active dictionary. */
export function useT(): Dictionary {
  return useContext(TranslationContext);
}
