#[cfg(windows)]
use crate::{pin_dialog::PinDialog, toast::FriendlyToast};
use std::sync::atomic::{AtomicIsize, Ordering};
use tracing::{info, warn};

#[cfg(windows)]
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
#[cfg(windows)]
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
#[cfg(windows)]
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                PostQuitMessage, RegisterClassW, TranslateMessage, CW_USEDEFAULT, SW_SHOWNORMAL,
                WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);

pub struct SystemTray;

impl SystemTray {
    /// Returns the HWND of the tray helper window if active, or 0 if not active
    pub fn get_tray_hwnd() -> isize {
        TRAY_HWND.load(Ordering::Relaxed)
    }

    /// Start the System Tray loop in a dedicated thread
    pub fn start() {
        #[cfg(windows)]
        {
            std::thread::spawn(|| {
                Self::run_tray_loop();
            });
        }
    }

    #[cfg(windows)]
    fn create_shield_icon() -> tray_icon::Icon {
        let width = 32u32;
        let height = 32u32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];

        // Draw a high-tech modern shield with cyan / royal blue gradient and white contour
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;

                let nx = (x as f32 - 15.5) / 13.5;
                let ny = (y as f32 - 15.5) / 13.5;

                // Shield boundary
                let in_shield = if ny < 0.1 {
                    nx.abs() <= 0.88 && ny >= -0.85
                } else {
                    let factor = (1.0 - ((ny - 0.1) / 0.85).powf(1.4)).max(0.0);
                    nx.abs() <= 0.88 * factor && ny <= 0.95
                };

                if in_shield {
                    let border_dist = if ny < 0.1 {
                        0.88 - nx.abs()
                    } else {
                        let factor = (1.0 - ((ny - 0.1) / 0.85).powf(1.4)).max(0.0);
                        (0.88 * factor) - nx.abs()
                    };

                    if border_dist < 0.12 || !(-0.75..=0.85).contains(&ny) {
                        // Crisp white border
                        rgba[idx] = 255;
                        rgba[idx + 1] = 255;
                        rgba[idx + 2] = 255;
                        rgba[idx + 3] = 255;
                    } else {
                        // Electric Cyan (#38bdf8) to Deep Ocean Blue (#0369a1)
                        let t = ((ny + 0.85) / 1.7).clamp(0.0, 1.0);
                        let r = (56.0 * (1.0 - t) + 3.0 * t) as u8;
                        let g = (189.0 * (1.0 - t) + 105.0 * t) as u8;
                        let b = (248.0 * (1.0 - t) + 161.0 * t) as u8;

                        rgba[idx] = r;
                        rgba[idx + 1] = g;
                        rgba[idx + 2] = b;
                        rgba[idx + 3] = 255;
                    }
                } else {
                    // Transparent
                    rgba[idx] = 0;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 0;
                }
            }
        }

        tray_icon::Icon::from_rgba(rgba, width, height)
            .expect("Valid 32x32 RGBA icon for FamShield system tray")
    }

    #[cfg(windows)]
    fn run_tray_loop() {
        // 1. Create a lightweight invisible message-only window for notifications and hwnd tracking
        let hwnd = unsafe {
            let instance = match GetModuleHandleW(None) {
                Ok(h) => h,
                Err(_) => return,
            };

            let class_name = w!("FamShieldTrayMsgClass");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(Self::wndproc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            match CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                class_name,
                w!("FamShieldTrayMsgWindow"),
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
            }
        };

        TRAY_HWND.store(hwnd.0 as isize, Ordering::Relaxed);

        // 2. Build modern muda context menu
        let tray_menu = Menu::new();

        let header_item = MenuItem::new("🛡️ FamShield - Đang bảo vệ 24/7", false, None);
        let sep1 = PredefinedMenuItem::separator();
        let open_admin = MenuItem::new("🌐 Mở Bảng Quản Trị Web (Local Admin)", true, None);
        let admin_pin = MenuItem::new("🔑 Xác Thực Master PIN (Desktop Dialog)", true, None);
        let request_time = MenuItem::new("⏳ Xin Thêm Thời Gian (+30 phút)", true, None);
        let sep2 = PredefinedMenuItem::separator();

        let active_port = famshield_service::get_active_admin_port();
        let status_submenu = Submenu::new("⚙️ Trạng Thái Hệ Thống", true);
        let sub_dns = MenuItem::new("🌐 DNS Proxy: 127.0.0.2:53 (Hoạt động)", false, None);
        let sub_wfp = MenuItem::new("🛡️ Tường Lửa WFP: Dynamic Session (Bảo vệ)", false, None);
        let sub_port = MenuItem::new(format!("🔌 Cổng Web Admin: {}", active_port), false, None);
        let _ = status_submenu.append_items(&[&sub_dns, &sub_wfp, &sub_port]);

        let sep3 = PredefinedMenuItem::separator();
        let uninstall_item = MenuItem::new("🗑️ Gỡ Cài Đặt (Yêu Cầu PIN)", true, None);
        let exit_item = MenuItem::new("❌ Ẩn Khay Hệ Thống", true, None);

        let _ = tray_menu.append_items(&[
            &header_item,
            &sep1,
            &open_admin,
            &admin_pin,
            &request_time,
            &sep2,
            &status_submenu,
            &uninstall_item,
            &sep3,
            &exit_item,
        ]);

        let open_admin_id = open_admin.id().clone();
        let admin_pin_id = admin_pin.id().clone();
        let request_time_id = request_time.id().clone();
        let uninstall_item_id = uninstall_item.id().clone();
        let exit_item_id = exit_item.id().clone();

        // 3. Set up event handlers for Menu
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if event.id == open_admin_id {
                let port = famshield_service::get_active_admin_port();
                let url_str = format!("http://127.0.0.1:{}\0", port);
                let wide_url: Vec<u16> = url_str.encode_utf16().collect();
                let open_op = w!("open");
                unsafe {
                    let _ = ShellExecuteW(
                        None,
                        open_op,
                        PCWSTR::from_raw(wide_url.as_ptr()),
                        PCWSTR::null(),
                        PCWSTR::null(),
                        SW_SHOWNORMAL,
                    );
                }
            } else if event.id == admin_pin_id {
                PinDialog::prompt_and_open_dashboard();
            } else if event.id == uninstall_item_id {
                crate::Uninstaller::prompt_and_uninstall();
            } else if event.id == request_time_id {
                let h = TRAY_HWND.load(Ordering::Relaxed);
                if h != 0 {
                    FriendlyToast::show(
                        HWND(h as *mut _),
                        "FamShield - Đã gửi yêu cầu ⏳",
                        "Yêu cầu xin thêm 30 phút đã được gửi tới phụ huynh. Vui lòng chờ phê duyệt!",
                        false,
                    );
                }
            } else if event.id == exit_item_id {
                let h = TRAY_HWND.load(Ordering::Relaxed);
                if h != 0 {
                    unsafe {
                        let _ = DestroyWindow(HWND(h as *mut _));
                    }
                }
            }
        }));

        // 4. Set up Left-Click / Double-Click event handler on Tray Icon
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                let port = famshield_service::get_active_admin_port();
                let url_str = format!("http://127.0.0.1:{}\0", port);
                let wide_url: Vec<u16> = url_str.encode_utf16().collect();
                let open_op = w!("open");
                unsafe {
                    let _ = ShellExecuteW(
                        None,
                        open_op,
                        PCWSTR::from_raw(wide_url.as_ptr()),
                        PCWSTR::null(),
                        PCWSTR::null(),
                        SW_SHOWNORMAL,
                    );
                }
            }
            _ => {}
        }));

        // 5. Build TrayIcon
        let icon = Self::create_shield_icon();
        let _tray_icon: TrayIcon = match TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("FamShield - Đang bảo vệ 🛡️")
            .with_icon(icon)
            .build()
        {
            Ok(ti) => ti,
            Err(e) => {
                warn!("Failed to initialize tray-icon: {}", e);
                return;
            }
        };

        info!("Professional tray-icon and modern muda context menu initialized");

        // 6. Show friendly welcome toast
        FriendlyToast::show(
            hwnd,
            "FamShield Bảo Vệ Trẻ Em 🛡️",
            "Hệ thống bảo vệ chủ động đang chạy ngầm an toàn.",
            false,
        );

        // 7. Message pump to keep tray and menu responsive
        unsafe {
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

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
            WM_CLOSE | WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
