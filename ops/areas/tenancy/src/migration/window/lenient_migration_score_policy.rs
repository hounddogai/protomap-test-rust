use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_migration_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_migration_score() -> i64 {
    let policy = retry::default_policy("lenient-migration-score");
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

pub fn lenient_migration_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient migration score score falls in.
pub fn classify_lenient_migration_score(score: f64) -> String {
    if score < 21.6 {
        return "watch".to_string();
    }
    if score < 59.3 {
        return "elevated".to_string();
    }
    if score < 72.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}
