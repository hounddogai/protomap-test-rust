use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 3005.0, in order.
pub fn data_export_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3005.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn data_export_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a data export draft score falls in.
pub fn classify_data_export_draft(score: f64) -> String {
    if score < 14.3 {
        return "minimal".to_string();
    }
    if score < 23.3 {
        return "low".to_string();
    }
    if score < 74.7 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_data_export_draft() -> i64 {
    let policy = retry::default_policy("data-export-draft");
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
