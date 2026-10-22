//! Authenticated Uninstallation Workflow for FamShield Session Helper.
//!
//! Provides a secure, PIN-protected uninstallation interface.
//! Prevents unauthorized uninstallation by children while allowing parents
//! to cleanly remove FamShield and restore network DNS settings to default.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{error, info, warn};

#[cfg(windows)]
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDesktopWindow,
            GetDlgItem, GetMessageW, GetWindowRect, GetWindowTextW, MessageBoxW, PostQuitMessage,
            RegisterClassW, SetForegroundWindow, ShowWindow, TranslateMessage, BS_DEFPUSHBUTTON,
            ES_CENTER, ES_PASSWORD, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, SW_SHOW, WM_CLOSE,
            WM_COMMAND, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_OVERLAPPED,
            WS_SYSMENU, WS_VISIBLE,
        },
    },
};

static UNINSTALL_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

const ID_EDIT_PIN: usize = 201;
const ID_BTN_OK: usize = 202;
const ID_BTN_CANCEL: usize = 203;
const ID_EDIT_OTP: usize = 204;

#[derive(Debug)]
enum UninstallOutcome {
    Success,
    WrongPin,
    RequiresOtp,
    InvalidOtp,
    Locked,
}

pub struct Uninstaller;

impl Uninstaller {
    /// Launches the Master PIN prompt dialog to authenticate uninstallation
    pub fn prompt_and_uninstall() {
        #[cfg(windows)]
        {
            if UNINSTALL_DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
                return;
            }

            let handle = std::thread::spawn(|| {
                unsafe {
                    Self::run_uninstall_dialog();
                }
                UNINSTALL_DIALOG_ACTIVE.store(false, Ordering::SeqCst);
            });

            let _ = handle.join();
        }
    }

    #[cfg(windows)]
    unsafe fn run_uninstall_dialog() {
        let instance = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(_) => return,
        };

        let class_name = w!("FamShieldUninstallDialogClass");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(16 as *mut core::ffi::c_void),
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let dlg_width = 440;
        let dlg_height = 250;

        let hwnd_desktop = GetDesktopWindow();
        let mut desk_rect = windows::Win32::Foundation::RECT::default();
        let _ = GetWindowRect(hwnd_desktop, &mut desk_rect);

        let pos_x = (desk_rect.right - dlg_width) / 2;
        let pos_y = (desk_rect.bottom - dlg_height) / 2;

        let hwnd = match CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            class_name,
            w!("FamShield - Xác Thực Gỡ Cài Đặt"),
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

        // Static prompt description
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Vui lòng nhập Master PIN Phụ Huynh để xác nhận gỡ bỏ.\nSau khi gỡ, toàn bộ cấu hình mạng DNS sẽ được khôi phục."),
            WS_CHILD | WS_VISIBLE,
            24,
            14,
            380,
            36,
            hwnd,
            None,
            instance,
            None,
        );

        // Label for PIN
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Mã Master PIN:"),
            WS_CHILD | WS_VISIBLE,
            24,
            62,
            140,
            20,
            hwnd,
            None,
            instance,
            None,
        );

        // Edit box for PIN
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    (ES_CENTER | ES_PASSWORD) as u32,
                ),
            170,
            59,
            220,
            26,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_PIN as _),
            instance,
            None,
        );

        // Label for OTP
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Mã 2FA OTP (nếu có):"),
            WS_CHILD | WS_VISIBLE,
            24,
            98,
            140,
            20,
            hwnd,
            None,
            instance,
            None,
        );

        // Edit box for OTP
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(ES_CENTER as u32),
            170,
            95,
            220,
            26,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_OTP as _),
            instance,
            None,
        );

        // Button OK
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Gỡ Cài Đặt"),
            WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            95,
            145,
            115,
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
            225,
            145,
            115,
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
                    let edit_pin_hwnd = GetDlgItem(hwnd, ID_EDIT_PIN as i32).unwrap_or_default();
                    let pin_len = GetWindowTextW(edit_pin_hwnd, &mut pin_buf);

                    let mut otp_buf = [0u16; 64];
                    let edit_otp_hwnd = GetDlgItem(hwnd, ID_EDIT_OTP as i32).unwrap_or_default();
                    let otp_len = GetWindowTextW(edit_otp_hwnd, &mut otp_buf);

                    if pin_len > 0 {
                        let entered_pin = String::from_utf16_lossy(&pin_buf[..pin_len as usize]);
                        let entered_otp = if otp_len > 0 {
                            let s = String::from_utf16_lossy(&otp_buf[..otp_len as usize]);
                            let trimmed = s.trim().to_string();
                            if trimmed.is_empty() {
                                None
                            } else {
                                Some(trimmed)
                            }
                        } else {
                            None
                        };

                        let port = famshield_service::get_active_admin_port();

                        match Self::send_uninstall_request(
                            port,
                            &entered_pin,
                            entered_otp.as_deref(),
                        ) {
                            Ok(UninstallOutcome::Success) => {
                                info!("Uninstall request authenticated successfully.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Xác thực Master PIN thành công!\n\nFamShield đang tiến hành khôi phục cấu hình mạng DNS về ban đầu và dọn dẹp hệ thống..."),
                                    w!("FamShield - Đang Gỡ Cài Đặt"),
                                    MB_ICONINFORMATION | MB_OK,
                                );
                                let _ = DestroyWindow(hwnd);
                            }
                            Ok(UninstallOutcome::RequiresOtp) => {
                                warn!("Master PIN ok, but 2FA OTP is required.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Tài khoản đang kích hoạt xác thực 2 lớp (2FA).\n\nVui lòng nhập thêm mã OTP 6 số từ ứng dụng Authenticator hoặc mã dự phòng vào ô 'Mã 2FA OTP' để xác nhận."),
                                    w!("Yêu Cầu Xác Thực 2FA"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                            Ok(UninstallOutcome::InvalidOtp) => {
                                warn!("2FA OTP or backup code invalid.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Mã OTP 2FA hoặc mã dự phòng không chính xác!\nVui lòng kiểm tra lại."),
                                    w!("Xác Thực 2FA Thất Bại"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                            Ok(UninstallOutcome::Locked) => {
                                warn!("Master PIN account locked due to failed attempts.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Tài khoản tạm thời bị khóa do nhập sai nhiều lần.\nVui lòng đợi hết thời gian khóa trước khi thử lại."),
                                    w!("Tài Khoản Tạm Khóa"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                            Ok(UninstallOutcome::WrongPin) => {
                                warn!("Master PIN verification failed for uninstall.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Mật khẩu Master PIN không chính xác!\nKhông thể thực hiện gỡ cài đặt FamShield."),
                                    w!("Xác Thực Thất Bại"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                            Err(e) => {
                                error!("Failed to send uninstallation request: {}", e);
                                MessageBoxW(
                                    hwnd,
                                    w!("Không thể gửi yêu cầu gỡ cài đặt đến FamShield Service.\nVui lòng đảm bảo dịch vụ đang chạy."),
                                    w!("Lỗi Kết Nối"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                        }
                    } else {
                        MessageBoxW(
                            hwnd,
                            w!("Vui lòng nhập Master PIN."),
                            w!("Cảnh báo"),
                            MB_ICONWARNING | MB_OK,
                        );
                    }
                } else if cmd_id == ID_BTN_CANCEL {
                    let _ = DestroyWindow(hwnd);
                }
                LRESULT(0)
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

    fn send_uninstall_request(
        port: u16,
        pin: &str,
        otp: Option<&str>,
    ) -> Result<UninstallOutcome, String> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .map_err(|e| format!("Không thể kết nối đến cổng {}: {}", port, e))?;
        stream.set_read_timeout(Some(Duration::from_secs(6))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(6))).ok();

        let escaped_pin = pin.replace('\\', "\\\\").replace('"', "\\\"");
        let body = if let Some(otp_val) = otp {
            let escaped_otp = otp_val.replace('\\', "\\\\").replace('"', "\\\"");
            format!(
                r#"{{"master_pin":"{}","otp_code":"{}"}}"#,
                escaped_pin, escaped_otp
            )
        } else {
            format!(r#"{{"master_pin":"{}"}}"#, escaped_pin)
        };

        let req = format!(
            "POST /api/system/uninstall HTTP/1.1\r\n\
             Host: 127.0.0.1:{}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            port,
            body.len(),
            body
        );

        stream
            .write_all(req.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);

        if response.contains("\"success\":true") {
            Ok(UninstallOutcome::Success)
        } else if response.contains("\"requires_otp\":true") {
            Ok(UninstallOutcome::RequiresOtp)
        } else if response.contains("\"invalid_otp\":true") {
            Ok(UninstallOutcome::InvalidOtp)
        } else if response.contains("\"locked\":true") || response.contains("429 Too Many Requests")
        {
            Ok(UninstallOutcome::Locked)
        } else if response.contains("403 Forbidden") || response.contains("\"success\":false") {
            Ok(UninstallOutcome::WrongPin)
        } else {
            Err("Phản hồi không xác định từ dịch vụ".to_string())
        }
    }
}
