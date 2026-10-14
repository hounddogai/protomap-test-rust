use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn legacy_policy_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a legacy policy agreement code in its canonical LPA- form.
pub fn normalize_legacy_policy_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPA-") {
        return clean.clone();
    }
    format!("LPA-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy policy agreement score falls in.
pub fn classify_legacy_policy_agreement(score: f64) -> String {
    if score < 32.0 {
        return "minimal".to_string();
    }
    if score < 66.8 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_policy_agreement() -> i64 {
    let policy = retry::default_policy("legacy-policy-agreement");
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
