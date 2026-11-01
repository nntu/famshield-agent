//! Real-time Active Foreground Window Tracker via Windows `SetWinEventHook`.
//!
//! Subscribes to `EVENT_SYSTEM_FOREGROUND` WinEvents on a dedicated Win32 message pump.
//! Consumers use the short-lived cache when fresh and retain polling as a fallback.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{error, info, warn};

#[cfg(windows)]
use windows::Win32::{
    Foundation::{CloseHandle, HWND},
    System::{
        ProcessStatus::GetProcessImageFileNameW,
        Threading::{GetCurrentThreadId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_QUIT,
            EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
            GetWindowTextW, GetWindowThreadProcessId,
        },
    },
};

/// Event details captured immediately when foreground application changes
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveWindowChangeEvent {
    pub process_name: String,
    pub window_title: String,
    pub pid: u32,
}

static LAST_ACTIVE_WINDOW: OnceLock<Arc<Mutex<Option<(ActiveWindowChangeEvent, std::time::Instant)>>>> = OnceLock::new();
static CALLBACKS: OnceLock<Arc<Mutex<Vec<Box<dyn Fn(&ActiveWindowChangeEvent) + Send + Sync>>>>> =
    OnceLock::new();
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static IS_RUNNING: AtomicBool = AtomicBool::new(false);

fn get_last_active_storage(
) -> &'static Arc<Mutex<Option<(ActiveWindowChangeEvent, std::time::Instant)>>> {
    LAST_ACTIVE_WINDOW.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn get_callbacks() -> &'static Arc<Mutex<Vec<Box<dyn Fn(&ActiveWindowChangeEvent) + Send + Sync>>>>
{
    CALLBACKS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

/// Retrieve the most recently cached active window event captured by WinEventHook.
pub fn get_cached_active_window() -> Option<ActiveWindowChangeEvent> {
    get_last_active_storage()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|(event, captured_at)| {
            (captured_at.elapsed() <= std::time::Duration::from_secs(10))
                .then(|| event.clone())
        }))
}

/// Register a listener callback to be invoked instantly when active foreground window changes.
pub fn on_active_window_changed<F>(callback: F)
where
    F: Fn(&ActiveWindowChangeEvent) + Send + Sync + 'static,
{
    if let Ok(mut cbs) = get_callbacks().lock() {
        cbs.push(Box::new(callback));
    }
}

#[cfg(windows)]
unsafe extern "system" fn foreground_win_event_callback(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND && !hwnd.0.is_null() {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return;
        }

        // Query window title
        let mut title_buf = [0u16; 512];
        let t_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = if t_len > 0 {
            String::from_utf16_lossy(&title_buf[..t_len as usize])
        } else {
            String::new()
        };

        // Query process executable name from PID
        let process_name =
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut img_buf = [0u16; 512];
                let len = GetProcessImageFileNameW(handle, &mut img_buf);
                let _ = CloseHandle(handle);
                if len > 0 {
                    let full_path = String::from_utf16_lossy(&img_buf[..len as usize]);
                    full_path
                        .rsplit('\\')
                        .next()
                        .unwrap_or(&full_path)
                        .to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

        let ev = ActiveWindowChangeEvent {
            process_name,
            window_title,
            pid,
        };

        // Store latest active window event
        if let Ok(mut storage) = get_last_active_storage().lock() {
            *storage = Some((ev.clone(), std::time::Instant::now()));
        }

        // Trigger registered callbacks
        if let Ok(cbs) = get_callbacks().lock() {
            for cb in cbs.iter() {
                cb(&ev);
            }
        }
    }
}

/// Guard handle managing the lifecycle of the active window WinEvent hook thread.
pub struct ActiveWindowHookGuard {
    is_active: Arc<AtomicBool>,
}

impl ActiveWindowHookGuard {
    pub fn stop(&self) {
        if self.is_active.swap(false, Ordering::SeqCst) {
            #[cfg(windows)]
            {
                let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
                if tid > 0 {
                    unsafe {
                        let _ = PostThreadMessageW(tid, WM_QUIT, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(0));
                    }
                }
            }
            #[cfg(not(windows))]
            IS_RUNNING.store(false, Ordering::SeqCst);
            info!("Active Window WinEvent Hook stopped.");
        }
    }
}

impl Drop for ActiveWindowHookGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Starts the event-driven active window hook on a dedicated Win32 message pump thread.
pub fn start_active_window_hook() -> ActiveWindowHookGuard {
    let is_active = Arc::new(AtomicBool::new(true));

    if IS_RUNNING.swap(true, Ordering::SeqCst) {
        warn!("ActiveWindowHook is already active.");
        is_active.store(false, Ordering::SeqCst);
        return ActiveWindowHookGuard { is_active };
    }

    #[cfg(windows)]
    {
        let is_active_clone = is_active.clone();
        let spawn_result = std::thread::Builder::new()
            .name("famshield-winevent-hook".to_string())
            .spawn(move || {
                unsafe {
                    let tid = GetCurrentThreadId();
                    HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

                    let hook = SetWinEventHook(
                        EVENT_SYSTEM_FOREGROUND,
                        EVENT_SYSTEM_FOREGROUND,
                        None,
                        Some(foreground_win_event_callback),
                        0,
                        0,
                        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                    );

                    if hook.0.is_null() {
                        error!("Failed to register SetWinEventHook for EVENT_SYSTEM_FOREGROUND.");
                        is_active_clone.store(false, Ordering::SeqCst);
                        IS_RUNNING.store(false, Ordering::SeqCst);
                        return;
                    }

                    info!("SetWinEventHook successfully registered for EVENT_SYSTEM_FOREGROUND (Thread ID: {}).", tid);

                    // WinEvent out-of-context hooks require a standard Win32 message loop to dispatch callbacks
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    let _ = UnhookWinEvent(hook);
                    info!("UnhookWinEvent successfully executed.");
                    HOOK_THREAD_ID.store(0, Ordering::SeqCst);
                    is_active_clone.store(false, Ordering::SeqCst);
                    IS_RUNNING.store(false, Ordering::SeqCst);
                }
            });
        if let Err(e) = spawn_result {
            error!("Failed to spawn WinEvent hook thread; foreground polling fallback remains active: {}", e);
            is_active.store(false, Ordering::SeqCst);
            IS_RUNNING.store(false, Ordering::SeqCst);
        }
    }

    #[cfg(not(windows))]
    {
        info!("SetWinEventHook active window tracker initialized (non-Windows stub mode).");
        // Seed mock window for non-Windows tests
        if let Ok(mut storage) = get_last_active_storage().lock() {
            *storage = Some((ActiveWindowChangeEvent {
                process_name: "explorer.exe".to_string(),
                window_title: "Windows Explorer".to_string(),
                pid: 1001,
            }, std::time::Instant::now()));
        }
    }

    ActiveWindowHookGuard { is_active }
}
