use crate::error::SessionError;
use jpeg_encoder::{ColorType, Encoder};
use tracing::{info, warn};

#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
    SRCCOPY,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetDesktopWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

pub struct SmartVisualVerifier {
    blur_radius: u32,
}

impl Default for SmartVisualVerifier {
    fn default() -> Self {
        Self::new(0) // Default to 0: clean, sharp capture without blur
    }
}

impl SmartVisualVerifier {
    pub fn new(blur_radius: u32) -> Self {
        Self { blur_radius }
    }

    /// Captures the interactive display via Win32 GDI BitBlt, scales down to 480p,
    /// applies in-RAM Gaussian/box blur if blur_radius > 0, and encodes as compressed JPEG.
    pub fn capture_frame(&self) -> Result<Vec<u8>, SessionError> {
        info!("Capturing trigger frame (blur_radius={})", self.blur_radius);

        let (src_w, src_h, bgra_pixels) = Self::capture_screen_pixels()?;

        let (target_w, target_h) = Self::calculate_target_dimensions(src_w, src_h);

        // Downscale to target dimensions and convert BGRA -> RGB
        let mut rgb =
            Self::downscale_and_convert_to_rgb(&bgra_pixels, src_w, src_h, target_w, target_h);

        // Apply fast 3-pass box blur only if configured
        if self.blur_radius > 0 {
            Self::apply_gaussian_blur(
                &mut rgb,
                target_w as usize,
                target_h as usize,
                self.blur_radius as usize,
            );
        }

        // Compress to JPEG (<150 KiB quota)
        let mut jpeg = Vec::with_capacity(32 * 1024);
        let encoder = Encoder::new(&mut jpeg, 65);
        encoder
            .encode(&rgb, target_w as u16, target_h as u16, ColorType::Rgb)
            .map_err(|e| SessionError::ScreenCapture(format!("JPEG encoding failed: {e}")))?;

        info!(
            "Generated frame: {}x{}, {} bytes",
            target_w,
            target_h,
            jpeg.len()
        );

        Ok(jpeg)
    }

    /// Legacy alias for capture_frame
    pub fn capture_blurred_frame(&self) -> Result<Vec<u8>, SessionError> {
        self.capture_frame()
    }

    #[cfg(windows)]
    fn capture_screen_pixels() -> Result<(u32, u32, Vec<u8>), SessionError> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            if width <= 0 || height <= 0 {
                warn!(
                    "GDI desktop metrics unavailable (width={}, height={}), generating fallback",
                    width, height
                );
                return Ok(Self::fallback_synthetic_frame(640, 480));
            }

            let hwnd = GetDesktopWindow();
            let hdc_screen = GetDC(hwnd);
            if hdc_screen.is_invalid() {
                warn!("Failed to obtain desktop DC, generating fallback canvas");
                return Ok(Self::fallback_synthetic_frame(width as u32, height as u32));
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_invalid() {
                ReleaseDC(hwnd, hdc_screen);
                return Err(SessionError::ScreenCapture(
                    "CreateCompatibleDC failed".into(),
                ));
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
            if hbitmap.is_invalid() {
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(hwnd, hdc_screen);
                return Err(SessionError::ScreenCapture(
                    "CreateCompatibleBitmap failed".into(),
                ));
            }

            let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));

            let blt_res = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // negative for top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default()],
            };

            let pixel_bytes_len = (width as usize) * (height as usize) * 4;
            let mut buffer = vec![0u8; pixel_bytes_len];

            let lines = GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Cleanup GDI objects
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_screen);

            if blt_res.is_err() || lines == 0 {
                warn!("BitBlt or GetDIBits failed, falling back to synthetic canvas");
                return Ok(Self::fallback_synthetic_frame(width as u32, height as u32));
            }

            Ok((width as u32, height as u32, buffer))
        }
    }

    #[cfg(not(windows))]
    fn capture_screen_pixels() -> Result<(u32, u32, Vec<u8>), SessionError> {
        Ok(Self::fallback_synthetic_frame(640, 480))
    }

    fn fallback_synthetic_frame(width: u32, height: u32) -> (u32, u32, Vec<u8>) {
        let mut buffer = vec![128u8; (width as usize) * (height as usize) * 4];
        // Create subtle diagnostic gradient
        for y in 0..height as usize {
            for x in 0..width as usize {
                let idx = (y * width as usize + x) * 4;
                buffer[idx] = (x % 256) as u8; // B
                buffer[idx + 1] = (y % 256) as u8; // G
                buffer[idx + 2] = ((x + y) % 256) as u8; // R
                buffer[idx + 3] = 255; // A
            }
        }
        (width, height, buffer)
    }

    fn calculate_target_dimensions(src_w: u32, src_h: u32) -> (u32, u32) {
        if src_w == 0 || src_h == 0 {
            return (640, 480);
        }
        let target_h = 480.min(src_h);
        let target_w = ((src_w as u64 * target_h as u64) / src_h as u64) as u32;
        (target_w.max(1), target_h.max(1))
    }

    fn downscale_and_convert_to_rgb(
        bgra: &[u8],
        src_w: u32,
        src_h: u32,
        dst_w: u32,
        dst_h: u32,
    ) -> Vec<u8> {
        let mut rgb = vec![0u8; (dst_w as usize) * (dst_h as usize) * 3];
        for y in 0..dst_h as usize {
            let src_y = (y * src_h as usize) / (dst_h as usize);
            for x in 0..dst_w as usize {
                let src_x = (x * src_w as usize) / (dst_w as usize);
                let src_idx = (src_y * src_w as usize + src_x) * 4;
                let dst_idx = (y * dst_w as usize + x) * 3;

                if src_idx + 2 < bgra.len() {
                    rgb[dst_idx] = bgra[src_idx + 2]; // R
                    rgb[dst_idx + 1] = bgra[src_idx + 1]; // G
                    rgb[dst_idx + 2] = bgra[src_idx]; // B
                }
            }
        }
        rgb
    }

    /// Fast 3-pass separable box blur approximating Gaussian filter (O(N) time).
    fn apply_gaussian_blur(rgb: &mut [u8], width: usize, height: usize, radius: usize) {
        if radius == 0 || width == 0 || height == 0 {
            return;
        }
        let r = radius.min(width.min(height) / 2).max(1);
        let mut temp = vec![0u8; rgb.len()];

        // 3 iterations of separable horizontal + vertical box blur
        for _ in 0..3 {
            // Horizontal blur from rgb -> temp
            Self::box_blur_horizontal(rgb, &mut temp, width, height, r);
            // Vertical blur from temp -> rgb
            Self::box_blur_vertical(&temp, rgb, width, height, r);
        }
    }

    fn box_blur_horizontal(src: &[u8], dst: &mut [u8], width: usize, height: usize, r: usize) {
        let window_size = 2 * r + 1;
        for y in 0..height {
            let row_offset = y * width * 3;

            let mut sum_r = 0usize;
            let mut sum_g = 0usize;
            let mut sum_b = 0usize;

            // Initialize window sum with clamped left pixels
            for i in 0..=r {
                let col = 0;
                let idx = row_offset + col * 3;
                let factor = if i == 0 { r + 1 } else { 0 };
                if factor > 0 {
                    sum_r += (src[idx] as usize) * factor;
                    sum_g += (src[idx + 1] as usize) * factor;
                    sum_b += (src[idx + 2] as usize) * factor;
                }
            }
            for i in 1..=r {
                let col = i.min(width - 1);
                let idx = row_offset + col * 3;
                sum_r += src[idx] as usize;
                sum_g += src[idx + 1] as usize;
                sum_b += src[idx + 2] as usize;
            }

            for x in 0..width {
                let dst_idx = row_offset + x * 3;
                dst[dst_idx] = (sum_r / window_size) as u8;
                dst[dst_idx + 1] = (sum_g / window_size) as u8;
                dst[dst_idx + 2] = (sum_b / window_size) as u8;

                // Slide window
                let next_x = (x + r + 1).min(width - 1);
                let prev_x = (x.saturating_sub(r)).min(width - 1);

                let next_idx = row_offset + next_x * 3;
                let prev_idx = row_offset + prev_x * 3;

                sum_r = sum_r + (src[next_idx] as usize) - (src[prev_idx] as usize);
                sum_g = sum_g + (src[next_idx + 1] as usize) - (src[prev_idx + 1] as usize);
                sum_b = sum_b + (src[next_idx + 2] as usize) - (src[prev_idx + 2] as usize);
            }
        }
    }

    fn box_blur_vertical(src: &[u8], dst: &mut [u8], width: usize, height: usize, r: usize) {
        let window_size = 2 * r + 1;
        for x in 0..width {
            let col_offset = x * 3;

            let mut sum_r = 0usize;
            let mut sum_g = 0usize;
            let mut sum_b = 0usize;

            // Initialize window sum with clamped top pixels
            let top_idx = col_offset;
            sum_r += (src[top_idx] as usize) * (r + 1);
            sum_g += (src[top_idx + 1] as usize) * (r + 1);
            sum_b += (src[top_idx + 2] as usize) * (r + 1);

            for i in 1..=r {
                let row = i.min(height - 1);
                let idx = row * width * 3 + col_offset;
                sum_r += src[idx] as usize;
                sum_g += src[idx + 1] as usize;
                sum_b += src[idx + 2] as usize;
            }

            for y in 0..height {
                let dst_idx = y * width * 3 + col_offset;
                dst[dst_idx] = (sum_r / window_size) as u8;
                dst[dst_idx + 1] = (sum_g / window_size) as u8;
                dst[dst_idx + 2] = (sum_b / window_size) as u8;

                // Slide window
                let next_y = (y + r + 1).min(height - 1);
                let prev_y = (y.saturating_sub(r)).min(height - 1);

                let next_idx = next_y * width * 3 + col_offset;
                let prev_idx = prev_y * width * 3 + col_offset;

                sum_r = sum_r + (src[next_idx] as usize) - (src[prev_idx] as usize);
                sum_g = sum_g + (src[next_idx + 1] as usize) - (src[prev_idx + 1] as usize);
                sum_b = sum_b + (src[next_idx + 2] as usize) - (src[prev_idx + 2] as usize);
            }
        }
    }
}
