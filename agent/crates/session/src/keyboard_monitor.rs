use famshield_core::keyword::{KeywordCategory, KeywordMatch, KeywordMatcher, KeywordSeverity};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Maximum characters kept in RAM for sliding window pattern matching.
pub const MAX_BUFFER_CAPACITY: usize = 128;

/// A detected violation event ready to be logged or alerted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordViolationAlert {
    pub pattern: String,
    pub category: KeywordCategory,
    pub severity: KeywordSeverity,
    pub app_name: String,
    pub snippet_masked: String,
}

/// In-memory sliding buffer for keystrokes.
/// Adheres to Privacy-by-Design:
/// - Never writes raw keystrokes to disk.
/// - Keeps only up to `MAX_BUFFER_CAPACITY` characters in RAM.
/// - Clears automatically upon violation match or timeout.
#[derive(Debug)]
pub struct KeystrokeSlidingBuffer {
    buffer: VecDeque<char>,
    last_keystroke: Instant,
    active_app: String,
}

impl Default for KeystrokeSlidingBuffer {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}

impl KeystrokeSlidingBuffer {
    pub fn new(active_app: String) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_BUFFER_CAPACITY),
            last_keystroke: Instant::now(),
            active_app,
        }
    }

    pub fn set_active_app(&mut self, app_name: String) {
        if self.active_app != app_name {
            self.active_app = app_name;
            // Clear buffer on app switch to prevent cross-app keyword concatenation
            self.buffer.clear();
        }
    }

    pub fn active_app(&self) -> &str {
        &self.active_app
    }

    /// Appends a typed character. If capacity is exceeded, drops oldest char.
    pub fn push_char(&mut self, ch: char) {
        self.last_keystroke = Instant::now();
        if self.buffer.len() >= MAX_BUFFER_CAPACITY {
            self.buffer.pop_front();
        }
        self.buffer.push_back(ch);
    }

    /// Handles Backspace key by removing the last character.
    pub fn handle_backspace(&mut self) {
        self.last_keystroke = Instant::now();
        self.buffer.pop_back();
    }

    /// Clears the buffer (e.g. after violation detection or timeout).
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the current accumulated text in the window.
    pub fn current_text(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Checks if user has been idle for more than idle_timeout_secs and clears buffer if so.
    pub fn check_idle_timeout(&mut self, idle_timeout_secs: u64) {
        if self.last_keystroke.elapsed().as_secs() >= idle_timeout_secs {
            self.buffer.clear();
        }
    }
}

/// Evaluator that connects the sliding window buffer with the Aho-Corasick KeywordMatcher.
pub struct KeyboardMonitor {
    buffer: Arc<Mutex<KeystrokeSlidingBuffer>>,
    matcher: Arc<Mutex<KeywordMatcher>>,
    enabled: AtomicBool,
}

impl Default for KeyboardMonitor {
    fn default() -> Self {
        Self::new(KeywordMatcher::default())
    }
}

impl KeyboardMonitor {
    pub fn new(matcher: KeywordMatcher) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(KeystrokeSlidingBuffer::default())),
            matcher: Arc::new(Mutex::new(matcher)),
            enabled: AtomicBool::new(true),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        if !enabled {
            if let Ok(mut buf) = self.buffer.lock() {
                buf.clear();
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_active_app(&self, app_name: &str) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.set_active_app(app_name.to_string());
        }
    }

    pub fn update_matcher(&self, new_matcher: KeywordMatcher) {
        if let Ok(mut m) = self.matcher.lock() {
            *m = new_matcher;
        }
    }

    /// Processes a keystroke character and immediately evaluates whether a violation occurred.
    /// If a match is found:
    /// 1. Flushes the buffer so the same keystroke sequence is not reported repeatedly.
    /// 2. Returns a `KeywordViolationAlert` with masked snippet.
    pub fn process_char(&self, ch: char) -> Option<KeywordViolationAlert> {
        if !self.is_enabled() {
            return None;
        }

        let mut buf = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return None,
        };

        buf.push_char(ch);
        let text = buf.current_text();

        let matcher = match self.matcher.lock() {
            Ok(m) => m,
            Err(_) => return None,
        };

        let matches = matcher.scan_text(&text);
        if let Some(first_match) = matches.into_iter().next() {
            let app_name = buf.active_app().to_string();
            // Privacy mask: do not record the full sentence, only a masked indicator
            let snippet_masked = format!("*** [{}] ***", first_match.pattern);

            // Flush buffer immediately
            buf.clear();

            Some(KeywordViolationAlert {
                pattern: first_match.pattern,
                category: first_match.category,
                severity: first_match.severity,
                app_name,
                snippet_masked,
            })
        } else {
            None
        }
    }

    /// Processes a Backspace key.
    pub fn process_backspace(&self) {
        if !self.is_enabled() {
            return;
        }
        if let Ok(mut buf) = self.buffer.lock() {
            buf.handle_backspace();
        }
    }

    /// Clears the internal keystroke buffer (e.g. on Enter, Tab, Escape, or context break).
    pub fn clear_buffer(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
    }

    /// Directly scans an arbitrary string (used for testing or batch inspection).
    pub fn test_scan(&self, text: &str) -> Vec<KeywordMatch> {
        if let Ok(m) = self.matcher.lock() {
            m.scan_text(text)
        } else {
            Vec::new()
        }
    }
}

/// Raw key event captured by the low-level hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawKeyEvent {
    pub vk_code: u32,
    pub scan_code: u32,
}

#[cfg(windows)]
static HOOK_HANDLE: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

#[cfg(windows)]
static KEY_EVENT_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<RawKeyEvent>> =
    std::sync::OnceLock::new();

/// Low-level keyboard hook callback.
/// CRITICAL: Must return immediately with minimal latency (<1ms).
/// Evaluates password field status before sending, drops if in password input.
#[cfg(windows)]
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    if n_code >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kbd_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        if let Some(tx) = KEY_EVENT_TX.get() {
            let _ = tx.try_send(RawKeyEvent {
                vk_code: kbd_struct.vkCode,
                scan_code: kbd_struct.scanCode,
            });
        }
    }

    let hook_val = HOOK_HANDLE.load(std::sync::atomic::Ordering::Relaxed);
    CallNextHookEx(HHOOK(hook_val as *mut _), n_code, wparam, lparam)
}

/// Checks whether the currently focused control is a password input field or password manager.
/// Adheres strictly to Privacy-by-Design:
/// 1. Queries Win32 GUI thread info for focused window style with `ES_PASSWORD (0x0020)`.
/// 2. Checks active foreground process name against known password managers / credential prompts.
#[cfg(windows)]
pub fn is_focused_password_field() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetGUIThreadInfo, GetWindowLongW, GetWindowThreadProcessId,
        GUITHREADINFO, GWL_STYLE,
    };

    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd.0.is_null() {
            return false;
        }

        // Fast check: Known password manager apps
        if let Some(proc_name) = crate::tracker::ForegroundTracker::get_foreground_process_name() {
            let lower = proc_name.to_lowercase();
            if lower.contains("credential")
                || lower.contains("keepass")
                || lower.contains("bitwarden")
                || lower.contains("1password")
                || lower.contains("lastpass")
                || lower.contains("enpass")
                || lower.contains("dashlane")
            {
                return true;
            }
        }

        let mut pid = 0u32;
        let thread_id = GetWindowThreadProcessId(fg_hwnd, Some(&mut pid));
        if thread_id == 0 {
            return false;
        }

        let mut gui_info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };

        if GetGUIThreadInfo(thread_id, &mut gui_info).is_ok() && !gui_info.hwndFocus.0.is_null() {
            let style = GetWindowLongW(gui_info.hwndFocus, GWL_STYLE) as u32;
            const ES_PASSWORD: u32 = 0x0020;
            if (style & ES_PASSWORD) != 0 {
                return true;
            }
        }
    }
    false
}

#[cfg(not(windows))]
pub fn is_focused_password_field() -> bool {
    false
}

/// Translates a virtual key code to a character.
#[cfg(windows)]
pub fn vk_to_char(vk_code: u32, scan_code: u32) -> Option<char> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, ToUnicode};

    let mut key_state = [0u8; 256];
    let mut char_buf = [0u16; 16];

    let count = unsafe {
        if GetKeyboardState(&mut key_state).is_ok() {
            ToUnicode(vk_code, scan_code, Some(&key_state), &mut char_buf, 0)
        } else {
            0
        }
    };

    if count > 0 {
        if let Some(ch) = char::decode_utf16(char_buf[..count as usize].iter().cloned())
            .next()
            .and_then(|r| r.ok())
        {
            if !ch.is_control() {
                return Some(ch);
            }
        }
    }

    // Direct mapping fallback for standard ASCII keys
    match vk_code {
        0x30..=0x39 => Some((b'0' + (vk_code as u8 - 0x30)) as char),
        0x41..=0x5A => Some((b'a' + (vk_code as u8 - 0x41)) as char),
        0x20 => Some(' '),
        0x60..=0x69 => Some((b'0' + (vk_code as u8 - 0x60)) as char), // NumPad 0-9
        0x6A => Some('*'),
        0x6B => Some('+'),
        0x6D => Some('-'),
        0x6E => Some('.'),
        0x6F => Some('/'),
        0xBA => Some(';'),
        0xBB => Some('='),
        0xBC => Some(','),
        0xBD => Some('-'),
        0xBE => Some('.'),
        0xBF => Some('/'),
        _ => None,
    }
}

#[cfg(not(windows))]
pub fn vk_to_char(vk_code: u32, _scan_code: u32) -> Option<char> {
    match vk_code {
        0x30..=0x39 => Some((b'0' + (vk_code as u8 - 0x30)) as char),
        0x41..=0x5A => Some((b'a' + (vk_code as u8 - 0x41)) as char),
        0x20 => Some(' '),
        _ => None,
    }
}

/// Retrieves the Windows session ID for current process.
#[cfg(windows)]
pub fn get_current_session_id() -> u32 {
    #[link(name = "kernel32")]
    extern "system" {
        fn WTSGetActiveConsoleSessionId() -> u32;
    }
    unsafe {
        let sid = WTSGetActiveConsoleSessionId();
        if sid == 0xFFFFFFFF {
            1
        } else {
            sid
        }
    }
}

#[cfg(not(windows))]
pub fn get_current_session_id() -> u32 {
    1
}

/// Dispatches detected violation to the Windows Service Session 0 over IPC Named Pipe.
pub async fn send_violation_via_ipc(
    session_id: u32,
    app_name: String,
    keyword_matched: String,
    category: String,
    severity: String,
    snippet_masked: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(windows)]
    {
        use famshield_service::ipc::{
            read_framed_message, write_framed_message, IpcMessage, IPC_PIPE_NAME,
        };
        use tokio::net::windows::named_pipe::ClientOptions;

        let message = IpcMessage::KeywordViolation {
            session_id,
            app_name,
            keyword_matched,
            category,
            severity,
            snippet_masked,
        };
        let mut last_error = None;
        for attempt in 0..3_u64 {
            let result = async {
                let mut client = ClientOptions::new().open(IPC_PIPE_NAME)?;
                write_framed_message(&mut client, &message).await?;
                match read_framed_message(&mut client).await? {
                    Some(IpcMessage::Response { success: true, .. }) => Ok(()),
                    Some(IpcMessage::Response { message, .. }) => Err(std::io::Error::other(
                        format!("keyword violation rejected by service: {message}"),
                    )),
                    Some(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "unexpected IPC response for keyword violation",
                    )),
                    None => Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "service closed IPC before acknowledging keyword violation",
                    )),
                }
            }
            .await;

            match result {
                Ok(()) => return Ok(()),
                Err(error) => last_error = Some(error),
            }
            if attempt < 2 {
                tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1))).await;
            }
        }

        Err(Box::new(last_error.unwrap_or_else(|| {
            std::io::Error::other("keyword violation IPC failed without an error")
        })))
    }
    #[cfg(not(windows))]
    {
        let _ = (
            session_id,
            app_name,
            keyword_matched,
            category,
            severity,
            snippet_masked,
        );
        Ok(())
    }
}

/// Dispatches event-triggered visual evidence (blurred screenshot in RAM) to Windows Service via IPC.
pub async fn send_visual_evidence_via_ipc(
    event_id: String,
    trigger: String,
    image_jpeg_bytes: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(windows)]
    {
        use famshield_service::ipc::{
            read_framed_message, write_framed_message, IpcMessage, IPC_PIPE_NAME,
        };
        use tokio::net::windows::named_pipe::ClientOptions;

        let message = IpcMessage::VisualEvidenceSubmission {
            event_id,
            trigger,
            image_jpeg_bytes,
        };
        let mut client = ClientOptions::new().open(IPC_PIPE_NAME)?;
        write_framed_message(&mut client, &message).await?;
        match read_framed_message(&mut client).await? {
            Some(IpcMessage::Response { success: true, .. }) => Ok(()),
            Some(IpcMessage::Response { message, .. }) => {
                Err(std::io::Error::other(format!("Visual evidence rejected by service: {message}")).into())
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected response for visual evidence submission",
            )
            .into()),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (event_id, trigger, image_jpeg_bytes);
        Ok(())
    }
}

/// Sends periodic foreground heartbeat to Windows Service and receives possible capture commands.
pub async fn send_heartbeat_via_ipc(
    session_id: u32,
    executable_name: String,
    is_idle: bool,
    idle_duration_ms: u64,
) -> Result<Option<famshield_service::ipc::IpcMessage>, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(windows)]
    {
        use famshield_service::ipc::{
            read_framed_message, write_framed_message, IpcMessage, IPC_PIPE_NAME,
        };
        use tokio::net::windows::named_pipe::ClientOptions;

        let message = IpcMessage::ForegroundHeartbeat {
            session_id,
            executable_name,
            is_idle,
            idle_duration_ms,
        };
        let mut client = ClientOptions::new().open(IPC_PIPE_NAME)?;
        write_framed_message(&mut client, &message).await?;
        let resp = read_framed_message(&mut client).await?;
        Ok(resp)
    }
    #[cfg(not(windows))]
    {
        let _ = (session_id, executable_name, is_idle, idle_duration_ms);
        Ok(None)
    }
}

/// Synchronizes active keyword rules and enabled state from Windows Service via IPC.
pub async fn sync_keyword_rules_from_service(monitor: Arc<KeyboardMonitor>) {
    #[cfg(windows)]
    {
        use famshield_service::ipc::{
            read_framed_message, write_framed_message, IpcMessage, IPC_PIPE_NAME,
        };
        use tokio::net::windows::named_pipe::ClientOptions;

        let Ok(mut client) = ClientOptions::new().open(IPC_PIPE_NAME) else {
            return;
        };
        if write_framed_message(&mut client, &IpcMessage::GetKeywordRulesRequest)
            .await
            .is_err()
        {
            return;
        }
        if let Ok(Some(IpcMessage::GetKeywordRulesResponse { enabled, rules })) =
            read_framed_message(&mut client).await
        {
            monitor.set_enabled(enabled);
            let new_matcher = famshield_core::keyword::KeywordMatcher::new(rules);
            monitor.update_matcher(new_matcher);
            tracing::info!(
                "Synchronized keyword rules from service (enabled: {})",
                enabled
            );
        }
    }
    #[cfg(not(windows))]
    {
        let _ = monitor;
    }
}

/// Starts the Win32 low-level keyboard hook and spawns worker thread.
pub fn start_keyboard_monitor_hook(monitor: Arc<KeyboardMonitor>) {
    #[cfg(windows)]
    {
        use std::sync::mpsc::sync_channel;
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
            UnhookWindowsHookEx, MSG, WH_KEYBOARD_LL,
        };

        let (tx, rx) = sync_channel::<RawKeyEvent>(512);
        let _ = KEY_EVENT_TX.set(tx);
        let runtime_handle = tokio::runtime::Handle::current();

        // Spawn worker thread for asynchronous keystroke processing & matching
        let worker_monitor = monitor.clone();
        std::thread::spawn(move || {
            let session_id = get_current_session_id();
            while let Ok(event) = rx.recv() {
                if is_focused_password_field() {
                    continue;
                }
                if event.vk_code == 0x08 {
                    // VK_BACK
                    worker_monitor.process_backspace();
                } else if event.vk_code == 0x0D || event.vk_code == 0x1B || event.vk_code == 0x09 {
                    // VK_RETURN (Enter), VK_ESCAPE, VK_TAB - clear context buffer on boundary
                    worker_monitor.clear_buffer();
                } else if let Some(ch) = vk_to_char(event.vk_code, event.scan_code) {
                    if let Some(violation) = worker_monitor.process_char(ch) {
                        tracing::warn!(
                            "Sensitive keyword violation detected: pattern='{}', category={:?}, app='{}'",
                            violation.pattern,
                            violation.category,
                            violation.app_name
                        );

                        // Trigger friendly educational toast
                        let tray_hwnd = crate::tray::SystemTray::get_tray_hwnd();
                        crate::toast::FriendlyToast::notify_sensitive_keyword_reminder(
                            tray_hwnd,
                            &format!("{:?}", violation.category),
                        );

                        let v_app = violation.app_name.clone();
                        let v_pat = violation.pattern.clone();
                        let v_cat = format!("{:?}", violation.category);
                        let v_sev = format!("{:?}", violation.severity);
                        let v_snip = violation.snippet_masked.clone();

                        // Asynchronously send violation & capture blurred visual evidence in RAM
                        runtime_handle.spawn(async move {
                            if let Err(e) = send_violation_via_ipc(
                                session_id,
                                v_app.clone(),
                                v_pat.clone(),
                                v_cat.clone(),
                                v_sev,
                                v_snip,
                            )
                            .await
                            {
                                tracing::warn!("Failed to dispatch keyword violation IPC: {}", e);
                            }

                            // Capture trigger frame with privacy Gaussian blur (radius=15, 480p JPEG)
                            let verifier = crate::capture::SmartVisualVerifier::new(15);
                            let capture_res = tokio::task::spawn_blocking(move || {
                                verifier.capture_blurred_frame()
                            })
                            .await;

                            match capture_res {
                                Ok(Ok(jpeg_bytes)) => {
                                    let now_micros = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_micros();
                                    let event_id = format!("kw-ev-{}", now_micros);
                                    let trigger = format!("KEYWORD: {} ({}) in {}", v_pat, v_cat, v_app);
                                    if let Err(e) = send_visual_evidence_via_ipc(event_id, trigger, jpeg_bytes).await {
                                        tracing::warn!("Failed to dispatch keyword visual evidence IPC: {}", e);
                                    }
                                }
                                Ok(Err(e)) => {
                                    tracing::warn!("Failed to capture keyword evidence frame: {}", e);
                                }
                                Err(e) => {
                                    tracing::warn!("Spawn blocking error during screen capture: {}", e);
                                }
                            }
                        });
                    }
                }
            }
        });

        // Spawn Hook thread with Windows message loop
        std::thread::spawn(move || unsafe {
            let instance = match GetModuleHandleW(None) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("Failed to get module handle: {}", e);
                    return;
                }
            };
            let hook =
                match SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), instance, 0)
                {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::error!("Failed to register WH_KEYBOARD_LL hook: {}", e);
                        return;
                    }
                };

            HOOK_HANDLE.store(hook.0 as isize, std::sync::atomic::Ordering::Relaxed);
            tracing::info!("Win32 WH_KEYBOARD_LL hook registered successfully.");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
            HOOK_HANDLE.store(0, std::sync::atomic::Ordering::Relaxed);
            tracing::info!("Win32 WH_KEYBOARD_LL hook unregistered.");
        });
    }

    #[cfg(not(windows))]
    {
        let _ = monitor;
        tracing::info!("Keyboard hook is not supported on non-Windows platforms.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_buffer_push_and_backspace() {
        let mut buf = KeystrokeSlidingBuffer::new("notepad.exe".to_string());
        buf.push_char('a');
        buf.push_char('b');
        buf.push_char('c');
        assert_eq!(buf.current_text(), "abc");

        buf.handle_backspace();
        assert_eq!(buf.current_text(), "ab");

        buf.push_char('z');
        assert_eq!(buf.current_text(), "abz");
    }

    #[test]
    fn test_sliding_buffer_app_switch_clears_buffer() {
        let mut buf = KeystrokeSlidingBuffer::new("discord.exe".to_string());
        buf.push_char('h');
        buf.push_char('e');
        buf.push_char('y');
        assert_eq!(buf.current_text(), "hey");

        buf.set_active_app("browser.exe".to_string());
        assert_eq!(buf.current_text(), "");
    }

    #[test]
    fn test_keyboard_monitor_detects_and_flushes() {
        let monitor = KeyboardMonitor::default();
        monitor.set_active_app("game.exe");

        // Type 'v', 'c', 'l'
        assert!(monitor.process_char('v').is_none());
        assert!(monitor.process_char('c').is_none());
        let violation = monitor.process_char('l');

        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.pattern, "vcl");
        assert_eq!(v.category, KeywordCategory::Profanity);
        assert_eq!(v.app_name, "game.exe");

        // Buffer should have been flushed automatically
        // Next key should not trigger repeat match
        assert!(monitor.process_char('!').is_none());
    }

    #[test]
    fn test_keyboard_monitor_detects_sex_in_browser() {
        let monitor = KeyboardMonitor::default();
        monitor.set_active_app("chrome.exe");

        // Type 's', 'e', 'x'
        assert!(monitor.process_char('s').is_none());
        assert!(monitor.process_char('e').is_none());
        let violation = monitor.process_char('x');

        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.pattern, "sex");
        assert_eq!(v.category, KeywordCategory::Adult);
        assert_eq!(v.app_name, "chrome.exe");
    }

    #[test]
    fn test_keyboard_monitor_disabled_does_nothing() {
        let monitor = KeyboardMonitor::default();
        monitor.set_enabled(false);

        for ch in "vcl".chars() {
            assert!(monitor.process_char(ch).is_none());
        }
    }

    #[test]
    fn test_raw_key_event_and_vk_to_char_translation() {
        let event = RawKeyEvent {
            vk_code: 0x41, // 'A'
            scan_code: 0x1E,
        };
        assert_eq!(event.vk_code, 0x41);

        // Test ASCII characters mapping fallback
        assert_eq!(vk_to_char(0x41, 0x1E), Some('a'));
        assert_eq!(vk_to_char(0x30, 0x0B), Some('0'));
        assert_eq!(vk_to_char(0x20, 0x39), Some(' '));
    }

    #[test]
    fn test_keyboard_monitor_detects_telex_and_vni_typing() {
        let monitor = KeyboardMonitor::default();
        monitor.set_active_app("browser.exe");

        // 1. Telex typing for single word: 'd', 'd', 'i', 'j', 't' (địt)
        for ch in "ddij".chars() {
            assert!(monitor.process_char(ch).is_none());
        }
        let alert = monitor.process_char('t');
        assert!(alert.is_some(), "Telex 'ddijt' should be detected");
        assert_eq!(alert.unwrap().pattern, "dit");

        // 2. VNI typing for single word: 'd', '9', 'i', '5', 't' (địt)
        for ch in "d9i5".chars() {
            assert!(monitor.process_char(ch).is_none());
        }
        let alert = monitor.process_char('t');
        assert!(alert.is_some(), "VNI 'd9i5t' should be detected");
        assert_eq!(alert.unwrap().pattern, "dit");

        // 3. Clear buffer on context break
        monitor.process_char('h');
        monitor.process_char('e');
        monitor.clear_buffer();
        // Subsequent keystrokes should not concatenate with previous 'he'
        assert_eq!(monitor.buffer.lock().unwrap().current_text(), "");
    }

    #[test]
    fn test_session_id_retrieval() {
        let session_id = get_current_session_id();
        assert!(session_id >= 1);
    }
}
