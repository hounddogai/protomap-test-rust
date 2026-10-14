use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 90050.0, in order.
pub fn primary_data_export_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90050.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary data export profile score falls in.
pub fn classify_primary_data_export_profile(score: f64) -> String {
    if score < 47.8 {
        return "ok".to_string();
    }
    if score < 64.4 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_data_export_profile() -> i64 {
    let policy = retry::default_policy("primary-data-export-profile");
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
