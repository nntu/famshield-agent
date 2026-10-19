#[cfg(windows)]
use crate::{pin_dialog::PinDialog, toast::FriendlyToast};
#[cfg(windows)]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD,
                NIM_DELETE, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
                DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW,
                PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu,
                TranslateMessage, CW_USEDEFAULT, IDI_APPLICATION, MF_DISABLED, MF_GRAYED,
                MF_SEPARATOR, MF_STRING, SW_SHOWNORMAL, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
                TPM_RETURNCMD, WM_CLOSE, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
                WNDCLASSW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

const WM_TRAYICON: u32 = WM_USER + 100;

const IDM_OPEN_WEB_ADMIN: usize = 1000;
const IDM_ADMIN_PIN: usize = 1001;
const IDM_REQUEST_TIME: usize = 1002;
const IDM_STATUS_INFO: usize = 1003;
const IDM_EXIT_TRAY: usize = 1004;

use std::sync::atomic::{AtomicIsize, Ordering};

static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);

pub struct SystemTray;

impl SystemTray {
    /// Returns the HWND of the tray window if active, or 0 if not active
    pub fn get_tray_hwnd() -> isize {
        TRAY_HWND.load(Ordering::Relaxed)
    }

    /// Start the System Tray loop in a dedicated thread
    pub fn start() {
        #[cfg(windows)]
        {
            std::thread::spawn(|| unsafe {
                Self::run_tray_loop();
            });
        }
    }

    #[cfg(windows)]
    unsafe fn run_tray_loop() {
        let instance = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(_) => return,
        };

        let class_name = w!("FamShieldTrayClass");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let hwnd = match CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            class_name,
            w!("FamShieldTrayWindow"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        TRAY_HWND.store(hwnd.0 as isize, Ordering::Relaxed);

        let icon = match LoadIconW(None, IDI_APPLICATION) {
            Ok(i) => i,
            Err(_) => return,
        };

        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: icon,
            ..Default::default()
        };

        let tip = "FamShield - Đang bảo vệ 🛡️";
        let mut idx = 0;
        for c in tip.encode_utf16().take(127) {
            nid.szTip[idx] = c;
            idx += 1;
        }
        nid.szTip[idx] = 0;

        let _ = Shell_NotifyIconW(NIM_ADD, &nid);

        // Show welcome toast on first launch
        FriendlyToast::show(
            hwnd,
            "FamShield Bảo Vệ Trẻ Em",
            "Hệ thống bảo vệ chủ động đang chạy ngầm an toàn.",
            false,
        );

        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup tray icon on termination
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        TRAY_HWND.store(0, Ordering::Relaxed);
    }

    #[cfg(windows)]
    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TRAYICON => {
                let event = lparam.0 as u32;
                if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                    Self::show_context_menu(hwnd);
                }
                LRESULT(0)
            }
            WM_CLOSE | WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    #[cfg(windows)]
    unsafe fn show_context_menu(hwnd: HWND) {
        let menu = match CreatePopupMenu() {
            Ok(m) => m,
            Err(_) => return,
        };

        let _ = AppendMenuW(
            menu,
            MF_STRING,
            IDM_OPEN_WEB_ADMIN,
            w!("🌐 Mở Bảng Quản Trị Web (Local Admin)"),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            IDM_ADMIN_PIN,
            w!("🛡️ Xác thực Master PIN (Hộp thoại Desktop)"),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            IDM_REQUEST_TIME,
            w!("⏳ Xin thêm thời gian sử dụng"),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(
            menu,
            MF_STRING | MF_GRAYED | MF_DISABLED,
            IDM_STATUS_INFO,
            w!("ℹ️ Trạng thái: Đang bảo vệ (127.0.0.2:53)"),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            IDM_EXIT_TRAY,
            w!("❌ Đóng thông báo khay hệ thống"),
        );

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);

        let cmd = TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN | TPM_RETURNCMD,
            pt.x,
            pt.y,
            0,
            hwnd,
            None,
        );

        let _ = DestroyMenu(menu);

        match cmd.0 as usize {
            IDM_OPEN_WEB_ADMIN => {
                let url = w!("http://127.0.0.1:48123");
                let open_op = w!("open");
                let _ = ShellExecuteW(
                    None,
                    open_op,
                    url,
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );
            }
            IDM_ADMIN_PIN => {
                PinDialog::prompt_and_open_dashboard();
            }
            IDM_REQUEST_TIME => {
                FriendlyToast::show(
                    hwnd,
                    "FamShield - Đã gửi yêu cầu ⏳",
                    "Yêu cầu xin thêm 30 phút đã được gửi tới phụ huynh. Vui lòng chờ phê duyệt!",
                    false,
                );
            }
            IDM_EXIT_TRAY => {
                let _ = DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}
