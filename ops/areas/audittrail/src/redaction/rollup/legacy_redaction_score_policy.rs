use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Counts how many amounts fit in a budget of 32467.0, in order.
pub fn legacy_redaction_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32467.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_redaction_score() -> i64 {
    let policy = retry::default_policy("legacy-redaction-score");
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

/// Names the band a legacy redaction score score falls in.
pub fn classify_legacy_redaction_score(score: f64) -> String {
    if score < 18.0 {
        return "critical".to_string();
    }
    if score < 34.6 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn legacy_redaction_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
