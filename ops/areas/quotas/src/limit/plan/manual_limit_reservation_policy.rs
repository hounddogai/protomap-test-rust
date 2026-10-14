use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a manual limit reservation score falls in.
pub fn classify_manual_limit_reservation(score: f64) -> String {
    if score < 38.3 {
        return "watch".to_string();
    }
    if score < 42.9 {
        return "ok".to_string();
    }
    if score < 66.2 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Puts a manual limit reservation code in its canonical MLR- form.
pub fn normalize_manual_limit_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MLR-") {
        return clean.clone();
    }
    format!("MLR-{}", clean.replace(" ", "-"))
}

pub fn manual_limit_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_limit_reservation() -> i64 {
    let policy = retry::default_policy("manual-limit-reservation");
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
