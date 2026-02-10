use chrono::{NaiveDateTime, Utc};

pub trait Clock {
    fn now(&self) -> NaiveDateTime;
}

#[derive(Debug, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }
}

#[derive(Debug, Clone)]
pub struct FakeClock(pub NaiveDateTime);

impl Clock for FakeClock {
    fn now(&self) -> NaiveDateTime {
        self.0
    }
}
