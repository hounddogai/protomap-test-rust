use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_nonce_template() -> i64 {
    let policy = retry::default_policy("batched-nonce-template");
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

/// Puts a batched nonce template code in its canonical BNT- form.
pub fn normalize_batched_nonce_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BNT-") {
        return clean.clone();
    }
    format!("BNT-{}", clean.replace(" ", "-"))
}

pub fn batched_nonce_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.34;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched nonce template score falls in.
pub fn classify_batched_nonce_template(score: f64) -> String {
    if score < 19.9 {
        return "elevated".to_string();
    }
    if score < 76.5 {
        return "watch".to_string();
    }
    if score < 92.0 {
        return "medium".to_string();
    }
    if score < 93.2 {
        return "low".to_string();
    }
    "critical".to_string()
}
