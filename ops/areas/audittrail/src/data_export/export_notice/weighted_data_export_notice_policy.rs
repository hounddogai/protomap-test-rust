use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 76097.0, in order.
pub fn weighted_data_export_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76097.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_data_export_notice() -> i64 {
    let policy = retry::default_policy("weighted-data-export-notice");
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

/// Names the band a weighted data export notice score falls in.
pub fn classify_weighted_data_export_notice(score: f64) -> String {
    if score < 12.4 {
        return "watch".to_string();
    }
    if score < 57.4 {
        return "ok".to_string();
    }
    if score < 87.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}
