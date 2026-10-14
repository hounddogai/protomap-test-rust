use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a primary failover ledger score falls in.
pub fn classify_primary_failover_ledger(score: f64) -> String {
    if score < 33.3 {
        return "elevated".to_string();
    }
    if score < 74.2 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn primary_failover_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn primary_failover_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_failover_ledger() -> i64 {
    let policy = retry::default_policy("primary-failover-ledger");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
