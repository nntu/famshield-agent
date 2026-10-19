use std::time::{Duration, Instant};
use famshield_core::domain::ExecutableName;
use famshield_session::{capture::SmartVisualVerifier, tracker::ForegroundTracker};

#[test]
fn test_foreground_tracker_idle_detection() {
    let mut tracker = ForegroundTracker::new(300); // 300s = 5m threshold
    let now = Instant::now();

    let exe = ExecutableName::parse("browser.exe").unwrap();
    tracker.switch_active_window(exe, now);

    assert!(!tracker.is_user_idle(now));

    // 100s later with no input
    let t100 = now + Duration::from_secs(100);
    assert!(!tracker.is_user_idle(t100));

    // 350s later with no input -> Idle
    let t350 = now + Duration::from_secs(350);
    assert!(tracker.is_user_idle(t350));

    // User types something
    tracker.record_input(t350);
    assert!(!tracker.is_user_idle(t350));
}

#[test]
fn test_smart_visual_verifier_frame_generation() {
    let verifier = SmartVisualVerifier::new(15);
    let frame = verifier.capture_blurred_frame().expect("Frame capture should succeed");

    assert!(!frame.is_empty());
    // Verify valid JPEG SOI bytes
    assert_eq!(frame[0], 0xFF);
    assert_eq!(frame[1], 0xD8);
    // Verify frame is well under 150 KiB quota
    assert!(frame.len() < 150 * 1024);
}

#[test]
fn test_friendly_toast_and_pin_dialog_exports() {
    use famshield_session::{FriendlyToast, PinDialog, SystemTray};

    // Smoke check invoking warning without crashing on non-interactive/headless
    FriendlyToast::warn_time_remaining(0, 5);
    FriendlyToast::notify_quota_expired(0);
    FriendlyToast::notify_content_blocked(0, "game.example.com");
    FriendlyToast::notify_time_extended(0, 15);
}

