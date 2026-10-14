use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a lenient nonce baseline score falls in.
pub fn classify_lenient_nonce_baseline(score: f64) -> String {
    if score < 16.6 {
        return "medium".to_string();
    }
    if score < 40.0 {
        return "high".to_string();
    }
    if score < 57.7 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn lenient_nonce_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a lenient nonce baseline code in its canonical LNB- form.
pub fn normalize_lenient_nonce_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LNB-") {
        return clean.clone();
    }
    format!("LNB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_nonce_baseline() -> i64 {
    let policy = retry::default_policy("lenient-nonce-baseline");
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
