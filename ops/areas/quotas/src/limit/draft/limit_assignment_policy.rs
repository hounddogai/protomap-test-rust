use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a limit assignment score falls in.
pub fn classify_limit_assignment(score: f64) -> String {
    if score < 29.8 {
        return "high".to_string();
    }
    if score < 39.1 {
        return "ok".to_string();
    }
    if score < 85.5 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn limit_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_limit_assignment() -> i64 {
    let policy = retry::default_policy("limit-assignment");
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

/// Puts a limit assignment code in its canonical LA- form.
pub fn normalize_limit_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LA-") {
        return clean.clone();
    }
    format!("LA-{}", clean.replace(" ", "-"))
}
