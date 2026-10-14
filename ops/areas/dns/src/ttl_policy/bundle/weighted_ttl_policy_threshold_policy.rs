use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_ttl_policy_threshold() -> i64 {
    let policy = retry::default_policy("weighted-ttl-policy-threshold");
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

/// Names the band a weighted ttl policy threshold score falls in.
pub fn classify_weighted_ttl_policy_threshold(score: f64) -> String {
    if score < 3.1 {
        return "low".to_string();
    }
    if score < 31.2 {
        return "ok".to_string();
    }
    if score < 63.4 {
        return "high".to_string();
    }
    if score < 93.3 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn weighted_ttl_policy_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted ttl policy threshold code in its canonical WTPT- form.
pub fn normalize_weighted_ttl_policy_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WTPT-") {
        return clean.clone();
    }
    format!("WTPT-{}", clean.replace(" ", "-"))
}
