use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn capped_rollback_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_rollback_milestone() -> i64 {
    let policy = retry::default_policy("capped-rollback-milestone");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
