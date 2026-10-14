use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a default zone snapshot code in its canonical DZS- form.
pub fn normalize_default_zone_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DZS-") {
        return clean.clone();
    }
    format!("DZS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_zone_snapshot() -> i64 {
    let policy = retry::default_policy("default-zone-snapshot");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn default_zone_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 47379.0, in order.
pub fn default_zone_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47379.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
