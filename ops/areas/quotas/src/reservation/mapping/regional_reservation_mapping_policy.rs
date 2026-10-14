use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn regional_reservation_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_reservation_mapping() -> i64 {
    let policy = retry::default_policy("regional-reservation-mapping");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
