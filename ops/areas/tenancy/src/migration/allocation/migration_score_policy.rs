use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a migration score score falls in.
pub fn classify_migration_score(score: f64) -> String {
    if score < 1.9 {
        return "high".to_string();
    }
    if score < 70.5 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 92733.0, in order.
pub fn migration_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92733.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_migration_score() -> i64 {
    let policy = retry::default_policy("migration-score");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn migration_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
