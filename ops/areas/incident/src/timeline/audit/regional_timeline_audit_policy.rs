use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 49832.0, in order.
pub fn regional_timeline_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49832.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional timeline audit score falls in.
pub fn classify_regional_timeline_audit(score: f64) -> String {
    if score < 6.8 {
        return "low".to_string();
    }
    if score < 11.4 {
        return "ok".to_string();
    }
    if score < 81.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_timeline_audit() -> i64 {
    let policy = retry::default_policy("regional-timeline-audit");
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
