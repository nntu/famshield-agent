//! Parent-Authenticated Service Restart Dialog for FamShield.
//!
//! Provides a secure interface for parents to restart the background
//! FamShield Windows Service from the System Tray using Master PIN or 2FA OTP.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{error, info, warn};

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
                GetDlgItem, GetMessageW, GetWindowRect, GetWindowTextW, MessageBoxW, PostQuitMessage,
                RegisterClassW, SetForegroundWindow, ShowWindow, TranslateMessage, BS_DEFPUSHBUTTON,
                ES_CENTER, ES_PASSWORD, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, SW_HIDE, SW_SHOW,
                WM_CLOSE, WM_COMMAND, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CAPTION, WS_CHILD,
                WS_OVERLAPPED, WS_SYSMENU, WS_VISIBLE,
            },
        },
    },
};

static RESTART_DIALOG_ACTIVE: AtomicBool = AtomicBool::new(false);

const ID_EDIT_PIN: usize = 301;
const ID_EDIT_OTP: usize = 302;
const ID_BTN_RESTART: usize = 303;
const ID_BTN_CANCEL: usize = 304;

#[derive(Debug)]
pub enum RestartOutcome {
    Success(String),
    WrongPin,
    RequiresOtp,
    InvalidOtp,
    ServiceOffline,
    ConnectionError(String),
}

pub struct ServiceRestartDialog;

impl ServiceRestartDialog {
    /// Launches the Master PIN / OTP prompt dialog to authenticate service restart
    pub fn prompt_and_restart() {
        #[cfg(windows)]
        {
            if RESTART_DIALOG_ACTIVE.swap(true, Ordering::SeqCst) {
                return;
            }

            let handle = std::thread::spawn(|| {
                unsafe {
                    Self::run_dialog();
                }
                RESTART_DIALOG_ACTIVE.store(false, Ordering::SeqCst);
            });

            let _ = handle.join();
        }
    }

    #[cfg(windows)]
    unsafe fn run_dialog() {
        let instance = match GetModuleHandleW(None) {
            Ok(h) => h,
            Err(_) => return,
        };

        let class_name = w!("FamShieldRestartDialogClass");

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
            w!("FamShield - Khởi Động Lại Dịch Vụ"),
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

        // Header Label
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Nhập Master PIN hoặc Mã OTP của Phụ huynh để xác nhận:"),
            WS_CHILD | WS_VISIBLE,
            24,
            16,
            390,
            22,
            hwnd,
            None,
            instance,
            None,
        );

        // Label 1: Master PIN
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Master PIN (Key của cha mẹ):"),
            WS_CHILD | WS_VISIBLE,
            24,
            46,
            180,
            20,
            hwnd,
            None,
            instance,
            None,
        );

        // Input 1: Master PIN (Masked)
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
            210,
            43,
            190,
            26,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_PIN as _),
            instance,
            None,
        );

        // Label 2: 2FA OTP
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Mã 2FA OTP (nếu có bật 2FA):"),
            WS_CHILD | WS_VISIBLE,
            24,
            82,
            180,
            20,
            hwnd,
            None,
            instance,
            None,
        );

        // Input 2: 2FA OTP
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(ES_CENTER as u32),
            210,
            79,
            190,
            26,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_EDIT_OTP as _),
            instance,
            None,
        );

        // Note Label
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Dịch vụ ngầm FamShield sẽ khởi động lại và nạp lại cấu hình."),
            WS_CHILD | WS_VISIBLE,
            24,
            118,
            390,
            20,
            hwnd,
            None,
            instance,
            None,
        );

        // Button: Restart
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Khởi Động Lại"),
            WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(BS_DEFPUSHBUTTON as u32),
            90,
            155,
            120,
            32,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_RESTART as _),
            instance,
            None,
        );

        // Button: Cancel
        let _ = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("BUTTON"),
            w!("Hủy"),
            WS_CHILD | WS_VISIBLE,
            230,
            155,
            110,
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
                if cmd_id == ID_BTN_RESTART {
                    let mut pin_buf = [0u16; 64];
                    let edit_pin_hwnd = GetDlgItem(hwnd, ID_EDIT_PIN as i32).unwrap_or_default();
                    let pin_len = GetWindowTextW(edit_pin_hwnd, &mut pin_buf);

                    let mut otp_buf = [0u16; 64];
                    let edit_otp_hwnd = GetDlgItem(hwnd, ID_EDIT_OTP as i32).unwrap_or_default();
                    let otp_len = GetWindowTextW(edit_otp_hwnd, &mut otp_buf);

                    let entered_pin = if pin_len > 0 {
                        Some(String::from_utf16_lossy(&pin_buf[..pin_len as usize]))
                    } else {
                        None
                    };

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

                    if entered_pin.is_none() && entered_otp.is_none() {
                        MessageBoxW(
                            hwnd,
                            w!("Vui lòng nhập Master PIN hoặc Mã OTP 2FA của cha mẹ!"),
                            w!("Cảnh Báo"),
                            MB_ICONWARNING | MB_OK,
                        );
                        return LRESULT(0);
                    }

                    let port = famshield_service::get_active_admin_port();

                    match Self::send_restart_request(
                        port,
                        entered_pin.as_deref().unwrap_or_default(),
                        entered_otp.as_deref(),
                    ) {
                        Ok(RestartOutcome::Success(_msg)) => {
                            info!("Service restart authenticated successfully via HTTP API.");
                            MessageBoxW(
                                hwnd,
                                w!("Xác thực thành công!\n\nFamShield Service đang được khởi động lại trong giây lát..."),
                                w!("FamShield - Đang Khởi Động Lại"),
                                MB_ICONINFORMATION | MB_OK,
                            );
                            let _ = DestroyWindow(hwnd);
                        }
                        Ok(RestartOutcome::RequiresOtp) => {
                            warn!("Master PIN valid, but 2FA OTP is required.");
                            MessageBoxW(
                                hwnd,
                                w!("Tài khoản đang bật xác thực 2 lớp (2FA).\n\nVui lòng nhập thêm mã OTP 6 số từ ứng dụng Authenticator hoặc mã dự phòng vào ô 'Mã 2FA OTP'."),
                                w!("Yêu Cầu Xác Thực 2FA"),
                                MB_ICONWARNING | MB_OK,
                            );
                        }
                        Ok(RestartOutcome::InvalidOtp) => {
                            warn!("2FA OTP or backup code is invalid.");
                            MessageBoxW(
                                hwnd,
                                w!("Mã OTP 2FA hoặc mã dự phòng không chính xác!\nVui lòng kiểm tra lại."),
                                w!("Lỗi Xác Thực"),
                                MB_ICONWARNING | MB_OK,
                            );
                        }
                        Ok(RestartOutcome::WrongPin) => {
                            warn!("Master PIN is incorrect.");
                            MessageBoxW(
                                hwnd,
                                w!("Master PIN không chính xác!\nVui lòng kiểm tra lại mã PIN của phụ huynh."),
                                w!("Lỗi Xác Thực"),
                                MB_ICONWARNING | MB_OK,
                            );
                        }
                        Ok(RestartOutcome::ServiceOffline) => {
                            info!("Service HTTP is offline. Attempting cold start via Windows SCM...");
                            let mut started = famshield_windows::start_service("FamShieldService").is_ok();

                            // If direct start failed (e.g. Access Denied on standard non-admin account),
                            // request elevated UAC execution via ShellExecuteW 'runas'
                            if !started {
                                info!("Standard user SCM start restricted. Prompting UAC elevation...");
                                let runas_op = w!("runas");
                                let cmd_exe = w!("cmd.exe");
                                let params = w!("/C sc.exe start FamShieldService");
                                let res = ShellExecuteW(hwnd,
                                    runas_op,
                                    cmd_exe,
                                    params,
                                    PCWSTR::null(),
                                    SW_HIDE,
                                );
                                if res.0 as isize > 32 {
                                    started = true;
                                }
                            }

                            if started {
                                MessageBoxW(
                                    hwnd,
                                    w!("Dịch vụ FamShield Service đang dừng.\n\nĐã gửi lệnh khởi động dịch vụ tới Windows Service Control Manager thành công!"),
                                    w!("FamShield - Khởi Động Dịch Vụ"),
                                    MB_ICONINFORMATION | MB_OK,
                                );
                                let _ = DestroyWindow(hwnd);
                            } else {
                                error!("Failed to start service via SCM or UAC elevation.");
                                MessageBoxW(
                                    hwnd,
                                    w!("Không thể khởi động dịch vụ. Yêu cầu quyền Quản trị viên (Administrator) của Windows để tiếp tục."),
                                    w!("Lỗi Khởi Động Dịch Vụ"),
                                    MB_ICONWARNING | MB_OK,
                                );
                            }
                        }
                        Ok(RestartOutcome::ConnectionError(e)) => {
                            warn!("Connection error: {}. Trying SCM start...", e);
                            let _ = famshield_windows::start_service("FamShieldService");
                            MessageBoxW(
                                hwnd,
                                w!("Đã gửi lệnh khôi phục dịch vụ tới hệ thống!"),
                                w!("FamShield"),
                                MB_ICONINFORMATION | MB_OK,
                            );
                            let _ = DestroyWindow(hwnd);
                        }
                        Err(e) => {
                            error!("Error communicating with service: {}", e);
                            MessageBoxW(
                                hwnd,
                                w!("Đã xảy ra lỗi khi gửi yêu cầu khởi động lại."),
                                w!("Lỗi"),
                                MB_ICONWARNING | MB_OK,
                            );
                        }
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

    fn send_restart_request(
        port: u16,
        pin: &str,
        otp: Option<&str>,
    ) -> Result<RestartOutcome, String> {
        let mut stream = match TcpStream::connect(format!("127.0.0.1:{}", port)) {
            Ok(s) => s,
            Err(_) => return Ok(RestartOutcome::ServiceOffline),
        };
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
            "POST /api/system/service/restart HTTP/1.1\r\n\
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
            Ok(RestartOutcome::Success("Khởi động lại thành công".into()))
        } else if response.contains("\"requires_otp\":true") {
            Ok(RestartOutcome::RequiresOtp)
        } else if response.contains("\"invalid_otp\":true") {
            Ok(RestartOutcome::InvalidOtp)
        } else if response.contains("\"wrong_pin\":true") {
            Ok(RestartOutcome::WrongPin)
        } else {
            Ok(RestartOutcome::WrongPin)
        }
    }
}
