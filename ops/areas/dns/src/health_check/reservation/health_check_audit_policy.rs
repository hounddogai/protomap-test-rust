use super::*;

use acme_kit::retry;

/// Names the band a health check audit score falls in.
pub fn classify_health_check_audit(score: f64) -> String {
    if score < 17.8 {
        return "watch".to_string();
    }
    if score < 19.8 {
        return "high".to_string();
    }
    if score < 65.9 {
        return "elevated".to_string();
    }
    if score < 95.0 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 62586.0, in order.
pub fn health_check_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62586.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_health_check_audit() -> i64 {
    let policy = retry::default_policy("health-check-audit");
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
