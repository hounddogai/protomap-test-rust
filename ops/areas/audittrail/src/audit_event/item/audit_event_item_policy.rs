use super::*;

use acme_kit::retry;

/// Names the band a audit event item score falls in.
pub fn classify_audit_event_item(score: f64) -> String {
    if score < 28.1 {
        return "minimal".to_string();
    }
    if score < 30.2 {
        return "critical".to_string();
    }
    if score < 46.0 {
        return "ok".to_string();
    }
    if score < 80.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 73830.0, in order.
pub fn audit_event_item_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73830.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_audit_event_item() -> i64 {
    let policy = retry::default_policy("audit-event-item");
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
