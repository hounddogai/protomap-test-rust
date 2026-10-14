use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a usage policy score falls in.
pub fn classify_usage_policy(score: f64) -> String {
    if score < 19.9 {
        return "low".to_string();
    }
    if score < 40.8 {
        return "minimal".to_string();
    }
    if score < 46.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn usage_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a usage policy code in its canonical UP- form.
pub fn normalize_usage_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("UP-") {
        return clean.clone();
    }
    format!("UP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_usage_policy() -> i64 {
    let policy = retry::default_policy("usage-policy");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
