#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDesktopWindow,
                GetDlgItem, GetMessageW, GetWindowRect, GetWindowTextW, MessageBoxW,
                PostQuitMessage, RegisterClassW, SetForegroundWindow, ShowWindow, TranslateMessage,
                BS_DEFPUSHBUTTON, ES_CENTER, ES_PASSWORD, MB_ICONWARNING, MB_OK, SW_SHOW,
                SW_SHOWNORMAL, WM_CLOSE, WM_COMMAND, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CAPTION,
                WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
            },
        },
    },
};

static DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

const ID_EDIT_PIN: usize = 101;
const ID_BTN_OK: usize = 102;
const ID_BTN_CANCEL: usize = 103;

pub struct PinDialog;

impl PinDialog {
    /// Launch the quick Master PIN dialog on the active desktop session
    pub fn prompt_and_open_dashboard() {
        #[cfg(windows)]
        {
            if DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
                // Already open
                return;
            }

            std::thread::spawn(|| {
                unsafe {
                    Self::run_dialog();
                }
                DIALOG_ACTIVE.store(false, Ordering::SeqCst);
            });
        }
    }

    #[cfg(windows)]
    unsafe fn run_dialog() {
        let instance = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(_) => return,
        };

        let class_name = w!("FamShieldPinDialogClass");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(16 as *mut core::ffi::c_void), // COLOR_BTNFACE + 1
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let dlg_width = 380;
        let dlg_height = 190;

        let hwnd_desktop = GetDesktopWindow();
        let mut desk_rect = windows::Win32::Foundation::RECT::default();
        let _ = GetWindowRect(hwnd_desktop, &mut desk_rect);

        let pos_x = (desk_rect.right - dlg_width) / 2;
        let pos_y = (desk_rect.bottom - dlg_height) / 2;

        let hwnd = match CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            class_name,
            w!("FamShield - Xác thực Master PIN Phụ huynh"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            pos_x,
            pos_y,
            dlg_width,
            dlg_height,
            None,
            None,
            instance,
            None,
        ) {
            Ok(h) => h,
            Err(_) => return,
        };

        // Static prompt label
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Nhập Master PIN để mở Bảng Quản trị Cục bộ:"),
            WS_CHILD | WS_VISIBLE,
            24,
            20,
            330,
            24,
            hwnd,
            None,
            instance,
            None,
        );

        // Masked Password Edit Control
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    (ES_PASSWORD | ES_CENTER) as u32,
                ),
            80,
            55,
            210,
            30,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_PIN as _),
            instance,
            None,
        );

        // Button OK
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Xác nhận"),
            WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            80,
            105,
            100,
            32,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_OK as _),
            instance,
            None,
        );

        // Button Cancel
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Hủy"),
            WS_CHILD | WS_VISIBLE,
            190,
            105,
            100,
            32,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_CANCEL as _),
            instance,
            None,
        );

        let _ = SetForegroundWindow(hwnd);
        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    #[cfg(windows)]
    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_COMMAND => {
                let cmd_id = wparam.0 & 0xffff;
                if cmd_id == ID_BTN_OK {
                    let mut pin_buf = [0u16; 64];
                    let edit_hwnd = GetDlgItem(hwnd, ID_EDIT_PIN as i32).unwrap_or_default();
                    let len = GetWindowTextW(edit_hwnd, &mut pin_buf);

                    if len > 0 {
                        // Open local dashboard in default browser
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

                        let _ = DestroyWindow(hwnd);
                    } else {
                        let _ = MessageBoxW(
                            hwnd,
                            w!("Vui lòng nhập Master PIN trước khi tiếp tục!"),
                            w!("FamShield"),
                            MB_OK | MB_ICONWARNING,
                        );
                    }
                    LRESULT(0)
                } else if cmd_id == ID_BTN_CANCEL {
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
