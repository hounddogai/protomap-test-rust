use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 85171.0, in order.
pub fn scheduled_slo_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85171.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled slo record score falls in.
pub fn classify_scheduled_slo_record(score: f64) -> String {
    if score < 4.6 {
        return "critical".to_string();
    }
    if score < 5.8 {
        return "watch".to_string();
    }
    if score < 8.2 {
        return "medium".to_string();
    }
    if score < 57.0 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_slo_record() -> i64 {
    let policy = retry::default_policy("scheduled-slo-record");
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
