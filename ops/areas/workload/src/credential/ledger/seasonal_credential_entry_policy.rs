use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_credential_entry() -> i64 {
    let policy = retry::default_policy("seasonal-credential-entry");
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

pub fn seasonal_credential_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 80255.0, in order.
pub fn seasonal_credential_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 80255.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a seasonal credential entry score falls in.
pub fn classify_seasonal_credential_entry(score: f64) -> String {
    if score < 8.9 {
        return "low".to_string();
    }
    if score < 47.5 {
        return "critical".to_string();
    }
    if score < 95.3 {
        return "ok".to_string();
    }
    "medium".to_string()
}
