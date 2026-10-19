use famshield_core::domain::ExecutableName;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ForegroundTracker {
    current_executable: Option<ExecutableName>,
    last_switch: Instant,
    last_input_time: Instant,
    idle_threshold_seconds: u32,
}

impl ForegroundTracker {
    pub fn new(idle_threshold_seconds: u32) -> Self {
        let now = Instant::now();
        Self {
            current_executable: None,
            last_switch: now,
            last_input_time: now,
            idle_threshold_seconds,
        }
    }

    pub fn record_input(&mut self, now: Instant) {
        self.last_input_time = now;
    }

    pub fn is_user_idle(&self, now: Instant) -> bool {
        let elapsed = now.duration_since(self.last_input_time).as_secs();
        elapsed >= self.idle_threshold_seconds as u64
    }

    pub fn switch_active_window(&mut self, exe: ExecutableName, now: Instant) {
        self.current_executable = Some(exe);
        self.last_switch = now;
        self.last_input_time = now;
    }

    pub fn active_executable(&self) -> Option<&ExecutableName> {
        self.current_executable.as_ref()
    }
}
