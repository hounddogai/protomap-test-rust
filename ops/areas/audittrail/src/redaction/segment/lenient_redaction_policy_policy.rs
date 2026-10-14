use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a lenient redaction policy score falls in.
pub fn classify_lenient_redaction_policy(score: f64) -> String {
    if score < 2.9 {
        return "critical".to_string();
    }
    if score < 48.7 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_redaction_policy() -> i64 {
    let policy = retry::default_policy("lenient-redaction-policy");
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

/// Puts a lenient redaction policy code in its canonical LRP- form.
pub fn normalize_lenient_redaction_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRP-") {
        return clean.clone();
    }
    format!("LRP-{}", clean.replace(" ", "-"))
}

pub fn lenient_redaction_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.07;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
