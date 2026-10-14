//! Clocks, so code that reads the time can be tested.

use std::cell::Cell;
use std::rc::Rc;

/// Clock tells the current time in Unix seconds.
pub trait Clock: std::fmt::Debug {
    fn now(&self) -> i64;
}

/// SystemClock reads the wall clock.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default()
    }
}

/// FixedClock always tells the same time.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedClock {
    pub at: i64,
}

impl Clock for FixedClock {
    fn now(&self) -> i64 {
        self.at
    }
}

/// OffsetClock runs ahead of (or behind) another clock by a fixed number of seconds.
#[derive(Debug, Clone)]
pub struct OffsetClock {
    pub base: Rc<dyn Clock>,
    pub offset: i64,
}

impl Clock for OffsetClock {
    fn now(&self) -> i64 {
        self.base.now() + self.offset
    }
}

/// StepClock advances by step seconds every time it is read.
#[derive(Debug, Clone, PartialEq)]
pub struct StepClock {
    pub current: Cell<i64>,
    pub step: i64,
}

impl Clock for StepClock {
    fn now(&self) -> i64 {
        let at = self.current.get();
        let next = self.current.get() + self.step;
        self.current.set(next);
        at
    }
}

pub fn system() -> Rc<dyn Clock> {
    Rc::new(SystemClock)
}
