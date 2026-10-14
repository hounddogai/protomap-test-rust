use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_rule_bundle() -> i64 {
    let policy = retry::default_policy("pending-rule-bundle");
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

/// Puts a pending rule bundle code in its canonical PRB- form.
pub fn normalize_pending_rule_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRB-") {
        return clean.clone();
    }
    format!("PRB-{}", clean.replace(" ", "-"))
}

pub fn pending_rule_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.57;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending rule bundle score falls in.
pub fn classify_pending_rule_bundle(score: f64) -> String {
    if score < 34.0 {
        return "minimal".to_string();
    }
    if score < 72.4 {
        return "high".to_string();
    }
    "critical".to_string()
}
