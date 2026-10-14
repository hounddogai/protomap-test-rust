use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a redaction summary score falls in.
pub fn classify_redaction_summary(score: f64) -> String {
    if score < 5.5 {
        return "minimal".to_string();
    }
    if score < 6.4 {
        return "elevated".to_string();
    }
    if score < 44.8 {
        return "medium".to_string();
    }
    if score < 45.7 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Puts a redaction summary code in its canonical RS- form.
pub fn normalize_redaction_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RS-") {
        return clean.clone();
    }
    format!("RS-{}", clean.replace(" ", "-"))
}

pub fn redaction_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_summary() -> i64 {
    let policy = retry::default_policy("redaction-summary");
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
