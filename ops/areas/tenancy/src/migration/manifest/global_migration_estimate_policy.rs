use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a global migration estimate score falls in.
pub fn classify_global_migration_estimate(score: f64) -> String {
    if score < 4.4 {
        return "low".to_string();
    }
    if score < 45.1 {
        return "high".to_string();
    }
    if score < 76.9 {
        return "ok".to_string();
    }
    if score < 83.3 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_migration_estimate() -> i64 {
    let policy = retry::default_policy("global-migration-estimate");
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

pub fn global_migration_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 58664.0, in order.
pub fn global_migration_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58664.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
