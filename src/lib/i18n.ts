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
    read: "Read aloud",
    about: "About",
    idle: "idle",
    loading: "Loading",
    loadFailed: "Could not read your settings",
    retry: "Try again",
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
      "Press your dictation key in any app and talk; press it again and the text is transcribed, optionally rewritten by the active mode, then pasted where your cursor was.",
    accessibilityTitle: "Accessibility permission needed.",
    accessibilityBody:
      "Without it macOS blocks the paste keystroke, so text reaches your clipboard but never the app you are typing in.",
    start: "Start dictation",
    stop: "Stop & paste",
    working: "Working…",
    copyText: "Copy text",
    empty: "Nothing yet. Start talking and the words appear here as they are recognised.",
    // The verb, not just the key: a bare key name says nothing about whether it starts
    // a take or ends one.
    pressHint: "press once to start, press again to paste",
    cancelHint: "said the wrong thing? press to cancel and start over",
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
    sttTestSection: "Speech key check",
    sttTestHint: "Opens a real stream for a moment, so a rejected key shows up here.",
    sttTest: "Test key",
    sttTestOk: "OK — the provider accepted the key.",
    sttTestFailed: "Failed:",
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
    theme: "Appearance",
    themeHint: "Dark for night work, light for a bright desk.",
    themeLight: "Light",
    themeDark: "Dark",
    followSystem: "Follow system",
    shortcutsSection: "Shortcuts",
    toggle: "Start / stop",
    toggleHint: "Press once to record, again to paste.",
    cancel: "Cancel",
    cancelHint: "Throws the current take away.",
    nextMode: "Next mode",
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

  read: {
    title: "Read selected text aloud",
    offLede:
      "Highlight text anywhere and press a key — the app reads it back to you. Useful when there's a lot to get through and your eyes are tired.",
    offKeyNote: "Uses the API key you already saved — nothing new to sign up for.",
    enable: "Turn on read-aloud",
    onLede: "Highlight text anywhere, press {hotkey} to hear it.",
    readHotkey: "Read shortcut",
    enabledLabel: "On",
    voiceSection: "Voice",
    provider: "Service",
    providerReuseNote: "uses your existing key",
    appId: "App ID",
    credentials: "App ID + token",
    appIdHint: "Vbee needs both, from the same page of its console. Saved together.",
    model: "Model",
    modelHint: "Rarely needs changing — the default matches this provider's current TTS model/endpoint.",
    voice: "Voice",
    voiceLoading: "Loading…",
    voiceNone: "(no voices returned)",
    speed: "Speed",
    speedUnsupported: "This provider does not support a speed control.",
    preview: "▶ Try it",
    previewing: "Playing…",
    modesSection: "Read modes",
    addMode: "+ Add",
    modeListHint: "Click a mode to edit its prompt or hotkey.",
    modeAi: "AI",
    modeVerbatim: "verbatim",
    aiToggle: "Run an AI pass first",
    aiToggleHint: "Off reads the highlighted text exactly as-is.",
    promptHint: "Written for listening: short sentences, no bullet points, no markdown.",
    voiceCommandSection: "Give a spoken instruction",
    voiceCommandBody:
      'Highlight text, press {hotkey}, say what you want ("summarize this and read it to me"), press again, listen.',
    voiceCommandHotkey: "Shortcut",
    maxChars: "Refuse selections longer than",
    maxCharsHint: "Blocks an accidental whole-page selection before it reaches a paid API.",
    privacy:
      "Highlighted text is sent to the AI and text-to-speech providers selected above — nothing is saved and no read-aloud history is kept. When an app will not hand over its selection, the player offers to read your clipboard instead; that text is only ever sent if you press the button.",
  },

  player: {
    preparing: "Preparing…",
    pause: "Pause",
    resume: "Resume",
    close: "Stop",
    clipboardOffer: "Couldn't grab the highlighted text. On your clipboard:",
    readClipboard: "Read the clipboard",
    mode: "Mode",
    speedLabel: "Speed",
    speed: "Applies from the next sentence",
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
    supportSection: "Support this project",
    supportBody:
      "Tocky Voice is free and always will be. If it saves you time, you can chip in so the next thing gets built too — scan with any Vietnamese banking app.",
    coursesTitle: "Learn to build things like this",
    coursesBody: "ME Code teaches AI automation, VibeCode and AI agents, in Vietnamese.",
    coursesLink: "See the courses",
    beforeSection: "Before you hand this to someone",
    // Four separate facts, so they are four separate lines. As one paragraph this was
    // a wall of text at the exact moment someone is deciding whether to trust the app
    // with their voice — which is the moment it has to be readable.
    privacy: [
      {
        lead: "Your voice leaves this machine.",
        body: "It is streamed to the speech provider you picked. If the mode uses AI cleanup, the text goes on to the AI provider; if you use read-aloud, the highlighted text goes to the voice provider. All of it runs on the accounts whose keys you entered.",
      },
      {
        lead: "Keys stay on this machine.",
        body: "In a file only your user account can read — or in the system keychain, if you switch that on under Providers.",
      },
      {
        lead: "One call to GitHub each time the app starts,",
        body: "to see whether a new version is out. It sends the app version and your IP address. Turn it off in Settings.",
      },
      {
        lead: "The free credit shown above is the vendor's own figure",
        body: "and can change at any time — check their signup page.",
      },
    ],
  },

  update: {
    sectionTitle: "Updates",
    currentVersion: "Version",
    checkButton: "Check for updates",
    checking: "Checking…",
    upToDate: "You're on the latest version.",
    errorPrefix: "Couldn't check for updates:",
    newVersionAvailable: "A new version is available:",
    notesTitle: "What's new",
    installButton: "Update and restart",
    updateNow: "Update now",
    badgeNew: "NEW",
    downloadButton: "Download the update ↗",
    macOnlyReason:
      "This build isn't code-signed, so macOS can't safely replace it in place — download and install it by hand instead.",
    downloading: "Downloading…",
    installedRestarting: "Installed — restarting…",
    autoCheckLabel: "Check for updates automatically",
    autoCheckHint:
      "Checks GitHub once per launch. Turn this off and use the button above instead.",
    bannerSeeWhatsNew: "See what's new",
    bannerDismiss: "Dismiss",
  },

  overlay: {
    speak: "Start speaking…",
    // On screen at the moment someone is wondering how to finish, so it says what to
    // do with the key rather than only naming it. Kept short: this is a 480px HUD
    // glanced at mid-sentence, not the place to teach — that is the Dictate tab.
    stop: "press to stop and paste",
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
      "Press one key anywhere on your computer, talk, press it again, and what you said is typed into whatever app you are in. A few short steps and you are ready. First, which language should this app be in?",
    themePrompt: "And how should it look? This changes as you pick, so try both.",

    micTitle: "Check your microphone",
    micBody:
      "Everything else assumes your voice is reaching this app, so start here. Pick an input and say something — the bars below have to move before you continue.",
    micWaiting: "Say something…",
    micHeard: "Heard you. This microphone works.",
    micFailed: "Could not open this microphone:",
    micSilentHintMac:
      "Nothing yet? Try a different input above. If none of them move, check that System Settings → Privacy & Security → Microphone allows this app, and that the device is not muted.",
    micSilentHintOther:
      "Nothing yet? Try a different input above. If none of them move, check that your system lets apps use the microphone and that the device is not muted.",

    speechTitle: "Connect a speech service",
    speechBody:
      "Your voice is transcribed by an online service, so you need a free key from one of these. Pick one, get the key, paste it below.",
    getFreeKey: "Get a free key",
    keyStep1: "Open {provider} and sign up — no card needed for the free credit.",
    keyStep2: "Copy the key it gives you and paste it here.",
    keySaved: "{provider} key saved.",
    keyChecking: "Checking the key with {provider}…",
    keyWorks: "{provider} accepted the key. Speech recognition is ready.",
    keyRejected: "{provider} would not accept this key:",

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
    tryVideoCaption: "50-second walkthrough",
    tryBody: "Last check: press a key below, say a sentence, and watch it land.",
    keyToggle: "press once to start, again to stop and paste — from any app",
    keyCancel: "said the wrong thing? this key throws the take away — nothing gets pasted",
    tryPreviewHint: "Your words will appear here…",
    tryWaiting: "Waiting for you to talk…",
    trySuccess: "Got it — that's the whole app. Everywhere else, this pastes into whatever app has focus.",
    tryEmpty: "Heard nothing that turned into words. Try again, closer to the mic.",
    tryButton: "Test it",

    rerunTitle: "Setup walkthrough",
    rerunBody: "Run the four-step setup again — handy if something stopped working.",
    rerun: "Run setup again",

    readAloudTitle: "By the way — it reads too",
    readAloudBody:
      "Highlight text anywhere and press {hotkey}, and the app reads it back to you. Uses the key you just entered.",
    readAloudHint: "Handy for long documents when your eyes need a break.",
    readAloudLater: "Maybe later",
    readAloudEnable: "Turn it on, {hotkey}",
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
    no_audio_captured:
      "The microphone opened but stayed completely silent, so there was nothing to transcribe. Pick the right input under Dictate → Microphone and check it is not muted.",
    transcription_failed: "Transcription failed.",
    // Deliberately does not say where the words ended up. Paste restores the previous
    // clipboard, history can be switched off, and the output mode decides the rest — so
    // naming three places was wrong on the most ordinary path of all. What is always
    // true is that they were delivered and are still on this panel.
    transcription_incomplete:
      "The connection to the speech service broke mid-take. What had been recognised by then was kept and delivered — the last few seconds may be missing. The words are still on this panel: press Copy if you need them again.",
    nothing_selected:
      "Nothing is highlighted. Select some text anywhere, then try again.",
    selection_too_long:
      "That selection is longer than the read-aloud limit. Select a shorter piece, or raise the limit in the Read tab.",
    tts_failed: "Could not read that aloud.",
    no_tts_key: "No API key saved for the read-aloud provider. Add one in the Read tab.",
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

/**
 * The shape of `en`, with every string leaf widened to `string`.
 *
 * Recursive rather than two levels deep, so an entry can be a list or a nested object
 * when the content calls for it — the About tab's privacy notes are four
 * `{ lead, body }` pairs — while a missing or misspelled key stays a compile error.
 * Widening only the leaves is the point: `vi` has to match `en`'s structure exactly
 * without being forced to repeat its English wording as a literal type.
 */
type Translated<T> = T extends string
  ? string
  : { -readonly [K in keyof T]: Translated<T[K]> };

type Dictionary = { -readonly [S in keyof typeof en]: Translated<(typeof en)[S]> };

const vi: Dictionary = {
  nav: {
    dictate: "Đọc",
    modes: "Chế độ",
    providers: "Nhà cung cấp",
    hotkeys: "Cài đặt",
    history: "Lịch sử",
    read: "Đọc to",
    about: "Giới thiệu",
    idle: "chờ",
    loading: "Đang tải",
    loadFailed: "Không đọc được cài đặt",
    retry: "Thử lại",
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
      "Bấm phím nói ở bất kỳ ứng dụng nào rồi nói; bấm lần nữa là lời nói được chuyển thành chữ, có thể được AI viết lại theo chế độ đang chọn, rồi dán vào đúng chỗ con trỏ.",
    accessibilityTitle: "Cần quyền Accessibility.",
    accessibilityBody:
      "Thiếu quyền này macOS sẽ chặn thao tác dán, nên chữ chỉ vào clipboard chứ không vào được ứng dụng bạn đang gõ.",
    start: "Bắt đầu đọc",
    stop: "Dừng & dán",
    working: "Đang xử lý…",
    copyText: "Chép đoạn này",
    empty: "Chưa có gì. Cứ nói, chữ sẽ hiện ở đây ngay khi nhận dạng được.",
    pressHint: "bấm một lần để bắt đầu, bấm lại để dán",
    cancelHint: "lỡ nói sai? bấm để huỷ và nói lại từ đầu",
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
      "Nhận dạng giọng nói chạy qua kết nối streaming; phần AI tinh chỉnh là một lượt gọi sau đó. Cả hai đều dùng tài khoản của bạn.",
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
    sttTestSection: "Kiểm tra key giọng nói",
    sttTestHint: "Mở thử một kết nối thật trong chốc lát, key sai sẽ lộ ra ngay ở đây.",
    sttTest: "Thử key",
    sttTestOk: "OK — nhà cung cấp đã chấp nhận key.",
    sttTestFailed: "Thất bại:",
    testSection: "Kiểm tra kết nối",
    test: "Thử model",
    testing: "Đang thử…",
    storageSection: "Nơi lưu khoá",
    useKeychain: "Dùng keychain hệ thống",
    useKeychainHint:
      "Mặc định tắt: key nằm trong file chỉ tài khoản của bạn đọc được. Keychain an toàn hơn, nhưng bản chưa ký sẽ khiến macOS hỏi mật khẩu đăng nhập mỗi lần đọc — chỉ nên bật khi app đã được ký. Key hiện có sẽ tự chuyển sang.",
  },

  behaviour: {
    title: "Cài đặt",
    lede:
      "Ngôn ngữ, phím tắt, âm thanh, lịch sử và khởi động cùng máy. Phím tắt là toàn cục — dùng được từ bất kỳ ứng dụng nào.",
    generalSection: "Chung",
    uiLanguage: "Ngôn ngữ giao diện",
    uiLanguageHint: "Theo ngôn ngữ hệ thống, trừ khi bạn tự chọn.",
    theme: "Giao diện",
    themeHint: "Tối cho lúc làm đêm, sáng cho bàn làm việc nhiều ánh sáng.",
    themeLight: "Sáng",
    themeDark: "Tối",
    followSystem: "Theo hệ thống",
    shortcutsSection: "Phím tắt",
    toggle: "Bắt đầu / dừng",
    toggleHint: "Bấm lần đầu để thu, bấm lần nữa để dán.",
    cancel: "Huỷ",
    cancelHint: "Bỏ luôn lần thu đang chạy.",
    nextMode: "Chế độ kế tiếp",
    feedbackSection: "Phản hồi",
    sounds: "Âm báo",
    soundsHint:
      "Tiếng bíp ngắn khi bắt đầu, dừng, xong và huỷ — để bạn đọc mà không cần nhìn màn hình.",
    volume: "Âm lượng",
    historySection: "Lịch sử",
    keepHistory: "Lưu lịch sử",
    keepHistoryHint: "Lưu trong file chỉ tài khoản của bạn đọc được.",
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
      "Mọi thứ bạn đã đọc, mới nhất trước — để lỡ dán hụt hay dán sai thì lấy lại được thay vì phải đọc lại.",
    empty: "Chưa có gì ở đây.",
    emptyHint: "Những lần đọc của bạn sẽ được lưu lại để chép lại sau.",
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

  read: {
    title: "Đọc văn bản bôi đen",
    offLede:
      "Bôi đen chữ ở bất kỳ đâu rồi bấm một phím — app đọc cho bạn nghe. Cần khi thông tin nhiều mà đọc bằng mắt thì mỏi.",
    offKeyNote: "Dùng chung API key bạn đã lưu, không phải đăng ký thêm gì.",
    enable: "Bật tính năng đọc",
    onLede: "Bôi đen chữ ở bất kỳ đâu, bấm {hotkey} để nghe.",
    readHotkey: "Phím tắt đọc",
    enabledLabel: "Bật",
    voiceSection: "Giọng đọc",
    provider: "Dịch vụ",
    providerReuseNote: "dùng key sẵn có",
    appId: "App ID",
    credentials: "App ID + token",
    appIdHint: "Vbee cần cả hai, lấy cùng một trang trong console của họ. Lưu chung một lần.",
    model: "Model",
    modelHint: "Ít khi cần đổi — giá trị mặc định khớp với model/endpoint TTS hiện tại của dịch vụ này.",
    voice: "Giọng",
    voiceLoading: "Đang tải…",
    voiceNone: "(không nạp được danh sách)",
    speed: "Tốc độ",
    speedUnsupported: "Nhà cung cấp này không chỉnh được tốc độ.",
    preview: "▶ Nghe thử",
    previewing: "Đang phát…",
    modesSection: "Chế độ đọc",
    addMode: "+ Thêm",
    modeListHint: "Bấm vào một chế độ để sửa prompt / gán phím tắt.",
    modeAi: "AI",
    modeVerbatim: "không AI",
    aiToggle: "Cho AI xử lý trước",
    aiToggleHint: "Tắt thì đọc đúng nguyên văn bôi đen.",
    promptHint: "Viết để nghe: câu ngắn, không gạch đầu dòng, không markdown.",
    voiceCommandSection: "Ra lệnh bằng giọng nói",
    voiceCommandBody:
      'Bôi đen → bấm {hotkey} → nói "tóm tắt rồi đọc cho tôi nghe" → bấm lại → nghe.',
    voiceCommandHotkey: "Phím tắt",
    maxChars: "Chặn khi bôi đen quá",
    maxCharsHint: "Chặn trường hợp lỡ bôi đen cả trang trước khi gọi API tốn tiền.",
    privacy:
      "Văn bản bôi đen được gửi tới nhà cung cấp AI và đọc mà bạn chọn ở trên — không lưu lại, không giữ lịch sử các lần đọc. Khi app nào đó không cho lấy chữ bôi đen, thanh điều khiển sẽ hỏi có đọc nội dung trong clipboard không; chỉ khi bạn bấm nút thì nội dung đó mới được gửi đi.",
  },

  player: {
    preparing: "Đang chuẩn bị…",
    pause: "Tạm dừng",
    resume: "Tiếp tục",
    close: "Dừng",
    clipboardOffer: "Không lấy được chữ bôi đen. Trong clipboard đang có:",
    readClipboard: "Đọc nội dung trong clipboard",
    mode: "Chế độ",
    speedLabel: "Tốc độ",
    speed: "Áp dụng từ câu kế tiếp",
  },

  about: {
    title: "Giới thiệu",
    lede:
      "Tocky Voice là mã nguồn mở. App dùng tài khoản nhà cung cấp của chính bạn, nên không ai đứng giữa thu tiền và không có âm thanh nào đi vòng qua bên thứ ba.",
    madeBy: "Thực hiện bởi",
    sttSection: "Nhận dạng giọng nói — chọn một",
    sttHint: "Bắt buộc. Đây là thứ biến giọng nói thành chữ.",
    llmSection: "AI tinh chỉnh — tuỳ chọn",
    llmHint:
      "Chỉ cần khi dùng chế độ có viết lại văn bản. Chế độ Raw chạy được mà không cần cái nào ở đây.",
    signUp: "Đăng ký ↗",
    getKey: "Lấy key ↗",
    supportSection: "Ủng hộ dự án",
    supportBody:
      "Tocky Voice miễn phí và sẽ luôn miễn phí. Nếu nó tiết kiệm thời gian cho bạn, có thể ủng hộ để tác giả làm tiếp sản phẩm sau — quét bằng app ngân hàng bất kỳ.",
    coursesTitle: "Học cách tự làm những thứ như thế này",
    coursesBody: "ME Code dạy AI Automation, VibeCode và AI Agent, bằng tiếng Việt.",
    coursesLink: "Xem các khoá học",
    beforeSection: "Trước khi đưa app này cho người khác",
    privacy: [
      {
        lead: "Giọng nói của bạn đi ra khỏi máy.",
        body: "Nó được truyền thẳng tới dịch vụ nhận dạng bạn chọn. Nếu chế độ có AI viết lại thì phần chữ đi tiếp tới nhà cung cấp AI; nếu bạn dùng đọc to thì đoạn bôi đen đi tới dịch vụ giọng đọc. Tất cả đều chạy bằng key của chính bạn.",
      },
      {
        lead: "Key nằm lại trên máy.",
        body: "Trong một file chỉ tài khoản của bạn đọc được — hoặc trong keychain hệ thống, nếu bạn bật ở tab Nhà cung cấp.",
      },
      {
        lead: "Mỗi lần mở app có một lần gọi tới GitHub",
        body: "để xem có bản mới không. Nó gửi đi phiên bản app và địa chỉ IP. Tắt được ở Cài đặt.",
      },
      {
        lead: "Con số credit miễn phí ở trên là hãng tự công bố,",
        body: "có thể đổi bất cứ lúc nào — kiểm tra lại ở trang đăng ký của họ.",
      },
    ],
  },

  update: {
    sectionTitle: "Cập nhật",
    currentVersion: "Phiên bản",
    checkButton: "Kiểm tra cập nhật",
    checking: "Đang kiểm tra…",
    upToDate: "Đang dùng bản mới nhất.",
    errorPrefix: "Không kiểm tra được:",
    newVersionAvailable: "Có bản mới:",
    notesTitle: "Có gì mới",
    installButton: "Cập nhật và khởi động lại",
    updateNow: "Cập nhật ngay",
    badgeNew: "MỚI",
    downloadButton: "Tải bản mới ↗",
    macOnlyReason:
      "Bản này chưa được ký số nên macOS không tự thay thế an toàn được — hãy tải về và cài tay.",
    downloading: "Đang tải…",
    installedRestarting: "Đã cài xong — đang khởi động lại…",
    autoCheckLabel: "Tự động kiểm tra bản mới",
    autoCheckHint: "Kiểm tra GitHub mỗi lần mở app. Tắt thì dùng nút bên trên để kiểm tra tay.",
    bannerSeeWhatsNew: "Xem có gì mới",
    bannerDismiss: "Bỏ qua",
  },

  overlay: {
    speak: "Nói đi…",
    stop: "bấm để dừng và dán",
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
      "Bấm một phím ở bất kỳ đâu trên máy, nói, bấm lại, và chữ được gõ thẳng vào ứng dụng bạn đang mở. Vài bước ngắn là xong. Trước tiên, bạn muốn app hiển thị bằng ngôn ngữ nào?",
    themePrompt: "Còn nhìn thế nào cho dễ chịu? Chọn cái nào là đổi ngay, thử cả hai đi.",

    micTitle: "Kiểm tra micro",
    micBody:
      "Mọi bước sau đều mặc định là giọng bạn đã vào được app, nên hãy bắt đầu từ đây. Chọn một micro rồi nói thử — thanh sóng bên dưới phải nhảy thì mới đi tiếp được.",
    micWaiting: "Nói thử một câu…",
    micHeard: "Nghe thấy rồi. Micro này chạy tốt.",
    micFailed: "Không mở được micro này:",
    micSilentHintMac:
      "Chưa thấy gì? Thử đổi sang micro khác ở trên. Nếu không cái nào nhảy, kiểm tra System Settings → Privacy & Security → Microphone có cho phép app này không, và micro có đang bị tắt tiếng không.",
    micSilentHintOther:
      "Chưa thấy gì? Thử đổi sang micro khác ở trên. Nếu không cái nào nhảy, kiểm tra xem hệ điều hành có cho ứng dụng dùng micro không, và micro có đang bị tắt tiếng không.",

    speechTitle: "Kết nối dịch vụ nhận dạng giọng nói",
    speechBody:
      "Giọng của bạn được chuyển thành chữ bởi một dịch vụ online, nên bạn cần một key miễn phí. Chọn một nhà cung cấp, lấy key, dán vào ô bên dưới.",
    getFreeKey: "Lấy key miễn phí",
    keyStep1: "Mở {provider} và đăng ký — không cần thẻ để nhận credit miễn phí.",
    keyStep2: "Copy key họ cấp rồi dán vào đây.",
    keySaved: "Đã lưu key {provider}.",
    keyChecking: "Đang kiểm tra key với {provider}…",
    keyWorks: "{provider} đã chấp nhận key. Nhận dạng giọng nói sẵn sàng.",
    keyRejected: "{provider} không chấp nhận key này:",

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
    tryVideoCaption: "Video hướng dẫn 50 giây",
    tryBody: "Kiểm tra cuối: bấm 1 phím bên dưới, nói một câu, xem chữ hiện ra.",
    keyToggle: "bấm một lần để bắt đầu, bấm lại để dừng và dán — từ ứng dụng bất kỳ",
    keyCancel: "lỡ nói sai? phím này huỷ luôn lần thu — không có gì được dán cả",
    tryPreviewHint: "Chữ của bạn sẽ hiện ở đây…",
    tryWaiting: "Đang chờ bạn nói…",
    trySuccess: "Nhận được rồi — vậy là xong. Ở mọi ứng dụng khác, chữ sẽ dán vào đúng chỗ con trỏ.",
    tryEmpty: "Chưa nghe ra chữ nào. Thử lại, nói gần mic hơn.",
    tryButton: "Bấm để kiểm tra",

    rerunTitle: "Hướng dẫn cài đặt",
    rerunBody: "Chạy lại 4 bước cài đặt — hữu ích khi có gì đó ngừng hoạt động.",
    rerun: "Chạy lại hướng dẫn",

    readAloudTitle: "Nhân tiện — app còn đọc được",
    readAloudBody:
      "Bôi đen chữ ở bất kỳ đâu rồi bấm {hotkey}, app đọc cho bạn nghe. Dùng chung key bạn vừa nhập.",
    readAloudHint: "Hợp khi tài liệu dài mà mắt đã mỏi.",
    readAloudLater: "Để sau",
    readAloudEnable: "Bật, {hotkey}",
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
    no_audio_captured:
      "Micro có mở nhưng im hoàn toàn, nên không có gì để nhận dạng. Chọn đúng micro ở tab Đọc → Micro và kiểm tra xem nó có bị tắt tiếng không.",
    transcription_failed: "Nhận dạng giọng nói thất bại.",
    transcription_incomplete:
      "Kết nối tới dịch vụ nhận dạng bị đứt giữa chừng. Phần chữ đã nghe được vẫn được giữ và đã giao đi — có thể thiếu vài giây cuối. Chữ vẫn còn trên bảng này: bấm Chép nếu cần lấy lại.",
    nothing_selected: "Chưa bôi đen chữ nào. Bôi đen một đoạn bất kỳ rồi thử lại.",
    selection_too_long:
      "Đoạn bôi đen dài hơn ngưỡng cho phép. Bôi đen ít hơn, hoặc nâng ngưỡng ở tab Đọc.",
    tts_failed: "Không đọc được đoạn này.",
    no_tts_key: "Chưa lưu API key cho dịch vụ đọc. Thêm key ở tab Đọc.",
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
