#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    UI::Shell::{
        Shell_NotifyIconW, NOTIFYICONDATAW, NIIF_INFO, NIIF_WARNING, NIM_MODIFY, NIF_INFO,
    },
};


pub struct FriendlyToast;

impl FriendlyToast {
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
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_INFO;
        nid.dwInfoFlags = if is_warning { NIIF_WARNING } else { NIIF_INFO };

        Self::copy_to_u16_slice(title, &mut nid.szInfoTitle);
        Self::copy_to_u16_slice(message, &mut nid.szInfo);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
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
}
