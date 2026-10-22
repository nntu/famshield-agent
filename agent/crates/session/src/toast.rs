use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    UI::Shell::{
        Shell_NotifyIconW, NIF_INFO, NIIF_INFO, NIIF_WARNING, NIM_MODIFY, NOTIFYICONDATAW,
    },
};

static SUPPRESS_TOASTS: AtomicBool = AtomicBool::new(false);

pub struct FriendlyToast;

impl FriendlyToast {
    /// Configure silent monitoring mode (suppress all balloon/toast popups)
    pub fn set_silent_mode(silent: bool) {
        SUPPRESS_TOASTS.store(silent, Ordering::Relaxed);
    }

    /// Check if silent monitoring mode is active
    pub fn is_silent_mode() -> bool {
        SUPPRESS_TOASTS.load(Ordering::Relaxed)
    }

    /// Helper: populate fixed size UTF-16 array
    #[cfg(windows)]
    fn copy_to_u16_slice<const N: usize>(src: &str, dest: &mut [u16; N]) {
        let mut idx = 0;
        for c in src.encode_utf16().take(N - 1) {
            dest[idx] = c;
            idx += 1;
        }
        dest[idx] = 0;
    }

    /// Show toast notification using the existing Tray Icon HWND
    #[cfg(windows)]
    pub fn show(hwnd: HWND, title: &str, message: &str, is_warning: bool) {
        if Self::is_silent_mode() {
            tracing::debug!("Toast popup suppressed by silent monitoring mode: title='{}'", title);
            return;
        }

        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_INFO,
            dwInfoFlags: if is_warning { NIIF_WARNING } else { NIIF_INFO },
            ..Default::default()
        };

        Self::copy_to_u16_slice(title, &mut nid.szInfoTitle);
        Self::copy_to_u16_slice(message, &mut nid.szInfo);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    #[cfg(not(windows))]
    pub fn show(#[allow(unused)] hwnd: isize, #[allow(unused)] title: &str, #[allow(unused)] message: &str, #[allow(unused)] is_warning: bool) {
        if Self::is_silent_mode() {
            tracing::debug!("Toast popup suppressed by silent monitoring mode: title='{}'", title);
        }
    }

    /// 1. Sắp hết thời gian dùng máy (còn 5 phút)
    pub fn warn_time_remaining(#[allow(unused)] hwnd: isize, remaining_mins: u32) {
        #[cfg(windows)]
        {
            let title = "FamShield - Nhắc nhở thời gian ⏰";
            let msg = format!(
                "Thời gian sử dụng máy tính hôm nay chỉ còn {} phút. Hãy hoàn thành bài tập hoặc lưu lại trò chơi nhé!",
                remaining_mins
            );
            Self::show(HWND(hwnd as *mut _), title, &msg, false);
        }
    }

    /// 2. Hết thời gian sử dụng hôm nay
    pub fn notify_quota_expired(#[allow(unused)] hwnd: isize) {
        #[cfg(windows)]
        {
            let title = "FamShield - Hết thời gian sử dụng 🛑";
            let msg = "Đã hết thời gian sử dụng hôm nay. Hãy nghỉ ngơi để bảo vệ mắt hoặc xin thêm giờ từ phụ huynh nhé!";
            Self::show(HWND(hwnd as *mut _), title, msg, true);
        }
    }

    /// 3. Chặn an toàn trang web hoặc ứng dụng
    pub fn notify_content_blocked(#[allow(unused)] hwnd: isize, target_name: &str) {
        #[cfg(windows)]
        {
            let title = "FamShield - Chặn an toàn 🛡️";
            let msg = format!(
                "Nội dung '{}' không phù hợp với lứa tuổi và đã được tạm khóa an toàn theo quy định gia đình.",
                target_name
            );
            Self::show(HWND(hwnd as *mut _), title, &msg, true);
        }
    }

    /// 4. Đã duyệt cấp thêm giờ
    pub fn notify_time_extended(#[allow(unused)] hwnd: isize, added_mins: u32) {
        #[cfg(windows)]
        {
            let title = "FamShield - Đã cấp thêm giờ! 🎉";
            let msg = format!(
                "Phụ huynh đã duyệt cấp thêm {} phút sử dụng cho bạn. Chúc bạn học tập vui vẻ!",
                added_mins
            );
            Self::show(HWND(hwnd as *mut _), title, &msg, false);
        }
    }

    /// 5. Nhắc nhở từ ngữ nhạy cảm / xúc phạm
    pub fn notify_sensitive_keyword_reminder(#[allow(unused)] hwnd: isize, category: &str) {
        #[cfg(windows)]
        {
            if hwnd == 0 {
                return;
            }
            let title = "FamShield - Nhắc nhở lời nói văn minh 🌟";
            let msg = format!(
                "Bạn vừa gõ từ ngữ thuộc nhóm '{}'. Hãy giữ lời nói lịch sự và tôn trọng người khác khi giao tiếp nhé!",
                category
            );
            Self::show(HWND(hwnd as *mut _), title, &msg, true);
        }
    }
}
