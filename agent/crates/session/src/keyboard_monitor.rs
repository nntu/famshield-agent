use famshield_core::keyword::{KeywordCategory, KeywordMatch, KeywordMatcher, KeywordSeverity};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Maximum characters kept in RAM for sliding window pattern matching.
pub const MAX_BUFFER_CAPACITY: usize = 128;

/// A detected violation event ready to be logged or alerted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordViolationAlert {
    pub pattern: String,
    pub category: KeywordCategory,
    pub severity: KeywordSeverity,
    pub app_name: String,
    pub snippet_masked: String,
}

/// In-memory sliding buffer for keystrokes.
/// Adheres to Privacy-by-Design:
/// - Never writes raw keystrokes to disk.
/// - Keeps only up to `MAX_BUFFER_CAPACITY` characters in RAM.
/// - Clears automatically upon violation match or timeout.
#[derive(Debug)]
pub struct KeystrokeSlidingBuffer {
    buffer: VecDeque<char>,
    last_keystroke: Instant,
    active_app: String,
}

impl Default for KeystrokeSlidingBuffer {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}

impl KeystrokeSlidingBuffer {
    pub fn new(active_app: String) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_BUFFER_CAPACITY),
            last_keystroke: Instant::now(),
            active_app,
        }
    }

    pub fn set_active_app(&mut self, app_name: String) {
        if self.active_app != app_name {
            self.active_app = app_name;
            // Clear buffer on app switch to prevent cross-app keyword concatenation
            self.buffer.clear();
        }
    }

    pub fn active_app(&self) -> &str {
        &self.active_app
    }

    /// Appends a typed character. If capacity is exceeded, drops oldest char.
    pub fn push_char(&mut self, ch: char) {
        self.last_keystroke = Instant::now();
        if self.buffer.len() >= MAX_BUFFER_CAPACITY {
            self.buffer.pop_front();
        }
        self.buffer.push_back(ch);
    }

    /// Handles Backspace key by removing the last character.
    pub fn handle_backspace(&mut self) {
        self.last_keystroke = Instant::now();
        self.buffer.pop_back();
    }

    /// Clears the buffer (e.g. after violation detection or timeout).
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the current accumulated text in the window.
    pub fn current_text(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Checks if user has been idle for more than idle_timeout_secs and clears buffer if so.
    pub fn check_idle_timeout(&mut self, idle_timeout_secs: u64) {
        if self.last_keystroke.elapsed().as_secs() >= idle_timeout_secs {
            self.buffer.clear();
        }
    }
}

/// Evaluator that connects the sliding window buffer with the Aho-Corasick KeywordMatcher.
pub struct KeyboardMonitor {
    buffer: Arc<Mutex<KeystrokeSlidingBuffer>>,
    matcher: Arc<Mutex<KeywordMatcher>>,
    enabled: bool,
}

impl Default for KeyboardMonitor {
    fn default() -> Self {
        Self::new(KeywordMatcher::default())
    }
}

impl KeyboardMonitor {
    pub fn new(matcher: KeywordMatcher) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(KeystrokeSlidingBuffer::default())),
            matcher: Arc::new(Mutex::new(matcher)),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            if let Ok(mut buf) = self.buffer.lock() {
                buf.clear();
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_active_app(&self, app_name: &str) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.set_active_app(app_name.to_string());
        }
    }

    pub fn update_matcher(&self, new_matcher: KeywordMatcher) {
        if let Ok(mut m) = self.matcher.lock() {
            *m = new_matcher;
        }
    }

    /// Processes a keystroke character and immediately evaluates whether a violation occurred.
    /// If a match is found:
    /// 1. Flushes the buffer so the same keystroke sequence is not reported repeatedly.
    /// 2. Returns a `KeywordViolationAlert` with masked snippet.
    pub fn process_char(&self, ch: char) -> Option<KeywordViolationAlert> {
        if !self.enabled {
            return None;
        }

        let mut buf = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return None,
        };

        buf.push_char(ch);
        let text = buf.current_text();

        let matcher = match self.matcher.lock() {
            Ok(m) => m,
            Err(_) => return None,
        };

        let matches = matcher.scan_text(&text);
        if let Some(first_match) = matches.into_iter().next() {
            let app_name = buf.active_app().to_string();
            // Privacy mask: do not record the full sentence, only a masked indicator
            let snippet_masked = format!("*** [{}] ***", first_match.pattern);

            // Flush buffer immediately
            buf.clear();

            Some(KeywordViolationAlert {
                pattern: first_match.pattern,
                category: first_match.category,
                severity: first_match.severity,
                app_name,
                snippet_masked,
            })
        } else {
            None
        }
    }

    /// Processes a Backspace key.
    pub fn process_backspace(&self) {
        if !self.enabled {
            return;
        }
        if let Ok(mut buf) = self.buffer.lock() {
            buf.handle_backspace();
        }
    }

    /// Directly scans an arbitrary string (used for testing or batch inspection).
    pub fn test_scan(&self, text: &str) -> Vec<KeywordMatch> {
        if let Ok(m) = self.matcher.lock() {
            m.scan_text(text)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_buffer_push_and_backspace() {
        let mut buf = KeystrokeSlidingBuffer::new("notepad.exe".to_string());
        buf.push_char('a');
        buf.push_char('b');
        buf.push_char('c');
        assert_eq!(buf.current_text(), "abc");

        buf.handle_backspace();
        assert_eq!(buf.current_text(), "ab");

        buf.push_char('z');
        assert_eq!(buf.current_text(), "abz");
    }

    #[test]
    fn test_sliding_buffer_app_switch_clears_buffer() {
        let mut buf = KeystrokeSlidingBuffer::new("discord.exe".to_string());
        buf.push_char('h');
        buf.push_char('e');
        buf.push_char('y');
        assert_eq!(buf.current_text(), "hey");

        buf.set_active_app("browser.exe".to_string());
        assert_eq!(buf.current_text(), "");
    }

    #[test]
    fn test_keyboard_monitor_detects_and_flushes() {
        let monitor = KeyboardMonitor::default();
        monitor.set_active_app("game.exe");

        // Type 'v', 'c', 'l'
        assert!(monitor.process_char('v').is_none());
        assert!(monitor.process_char('c').is_none());
        let violation = monitor.process_char('l');

        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.pattern, "vcl");
        assert_eq!(v.category, KeywordCategory::Profanity);
        assert_eq!(v.app_name, "game.exe");

        // Buffer should have been flushed automatically
        // Next key should not trigger repeat match
        assert!(monitor.process_char('!').is_none());
    }

    #[test]
    fn test_keyboard_monitor_disabled_does_nothing() {
        let mut monitor = KeyboardMonitor::default();
        monitor.set_enabled(false);

        for ch in "vcl".chars() {
            assert!(monitor.process_char(ch).is_none());
        }
    }
}
