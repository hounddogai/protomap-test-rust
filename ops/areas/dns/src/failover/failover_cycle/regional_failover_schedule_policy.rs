use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a regional failover schedule code in its canonical RFS- form.
pub fn normalize_regional_failover_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFS-") {
        return clean.clone();
    }
    format!("RFS-{}", clean.replace(" ", "-"))
}

pub fn regional_failover_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn regional_failover_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_failover_schedule() -> i64 {
    let policy = retry::default_policy("regional-failover-schedule");
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
