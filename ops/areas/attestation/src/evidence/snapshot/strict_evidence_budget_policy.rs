use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn strict_evidence_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_evidence_budget() -> i64 {
    let policy = retry::default_policy("strict-evidence-budget");
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

/// Names the band a strict evidence budget score falls in.
pub fn classify_strict_evidence_budget(score: f64) -> String {
    if score < 6.0 {
        return "elevated".to_string();
    }
    if score < 50.8 {
        return "high".to_string();
    }
    if score < 56.5 {
        return "ok".to_string();
    }
    if score < 60.6 {
        return "medium".to_string();
    }
    "low".to_string()
}
