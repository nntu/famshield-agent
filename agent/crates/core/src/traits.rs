use std::time::Instant;
use chrono::{DateTime, Utc};

/// Abstract clock trait enabling deterministic time testing (DST, midnight quota reset).
pub trait Clock: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;
    fn monotonic_now(&self) -> Instant;
}

/// Standard production system clock.
#[derive(Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn monotonic_now(&self) -> Instant {
        Instant::now()
    }
}

/// Mock clock for unit testing schedules, quotas, and expiration.
#[derive(Clone)]
pub struct MockClock {
    utc: DateTime<Utc>,
    monotonic: Instant,
}

impl MockClock {
    pub fn new(utc: DateTime<Utc>) -> Self {
        MockClock {
            utc,
            monotonic: Instant::now(),
        }
    }

    pub fn advance(&mut self, duration: std::time::Duration) {
        self.utc = self.utc + chrono::Duration::from_std(duration).unwrap();
        self.monotonic = self.monotonic + duration;
    }

    pub fn set_utc(&mut self, utc: DateTime<Utc>) {
        self.utc = utc;
    }
}

impl Clock for MockClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.utc
    }

    fn monotonic_now(&self) -> Instant {
        self.monotonic
    }
}
