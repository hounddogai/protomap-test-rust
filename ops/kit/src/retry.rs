//! Retry policies with pluggable backoff.

use std::rc::Rc;

/// Backoff says how many milliseconds to wait before the next attempt.
pub trait Backoff: std::fmt::Debug {
    fn delay(&self, attempt: i64) -> i64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantBackoff {
    pub millis: i64,
}

impl Backoff for ConstantBackoff {
    fn delay(&self, attempt: i64) -> i64 {
        self.millis
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearBackoff {
    pub step: i64,
    pub cap: i64,
}

impl Backoff for LinearBackoff {
    fn delay(&self, attempt: i64) -> i64 {
        (self.step * (attempt + 1)).min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialBackoff {
    pub base: i64,
    pub factor: f64,
    pub cap: i64,
}

impl Backoff for ExponentialBackoff {
    fn delay(&self, attempt: i64) -> i64 {
        let mut delay = self.base as f64;
        for _ in 0..attempt {
            delay = delay * self.factor;
        }
        (delay as i64).min(self.cap)
    }
}

/// JitterBackoff spreads another backoff's delays by a stable pseudo-random amount.
#[derive(Debug, Clone)]
pub struct JitterBackoff {
    pub inner: Rc<dyn Backoff>,
    pub spread: i64,
    pub salt: String,
}

impl Backoff for JitterBackoff {
    fn delay(&self, attempt: i64) -> i64 {
        let base = self.inner.delay(attempt);
        if self.spread <= 0 {
            return base;
        }
        let noise = format!("{}:{}", self.salt, attempt)
            .bytes()
            .fold(0i64, |hash, byte| (hash * 31 + byte as i64) % 2_147_483_647)
            % self.spread;
        base + noise
    }
}

/// Policy retries an operation up to attempts times.
#[derive(Debug, Clone)]
pub struct Policy {
    pub attempts: i64,
    pub backoff: Rc<dyn Backoff>,
}

impl Policy {
    /// Calls op until it succeeds and returns how many attempts it took.
    pub fn run(&self, op: impl Fn(i64) -> bool) -> Result<i64, String> {
        let mut waited = 0;
        for attempt in 0..self.attempts {
            if op(attempt) {
                return Ok(attempt + 1);
            }
            waited += self.backoff.delay(attempt);
        }
        return Err(format!("gave up after {} attempts and {} ms", self.attempts, waited));
    }

    pub fn total_delay(&self) -> i64 {
        let mut total = 0;
        for attempt in 0..self.attempts {
            total += self.backoff.delay(attempt);
        }
        total
    }
}

pub fn default_policy(salt: &str) -> Policy {
    Policy {
        attempts: 5,
        backoff: Rc::new(JitterBackoff {
            inner: Rc::new(ExponentialBackoff {
                base: 100,
                factor: 2.0,
                cap: 5000,
            }),
            spread: 50,
            salt: salt.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_at_the_first_success() {
        let policy = Policy {
            attempts: 4,
            backoff: Rc::new(ConstantBackoff { millis: 10 }),
        };
        let used = match policy.run(|attempt: i64| attempt == 2) {
            Ok(value) => value,
            Err(_) => {
                panic!("unexpected empty result");
            }
        };
        assert_eq!(used, 3, "attempts");
        assert_eq!(policy.total_delay(), 40, "total delay");
    }
}
