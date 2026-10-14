use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a regional verifier summary score falls in.
pub fn classify_regional_verifier_summary(score: f64) -> String {
    if score < 10.8 {
        return "minimal".to_string();
    }
    if score < 13.5 {
        return "high".to_string();
    }
    if score < 59.5 {
        return "medium".to_string();
    }
    if score < 67.7 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn regional_verifier_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_verifier_summary() -> i64 {
    let policy = retry::default_policy("regional-verifier-summary");
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

/// Puts a regional verifier summary code in its canonical RVS- form.
pub fn normalize_regional_verifier_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RVS-") {
        return clean.clone();
    }
    format!("RVS-{}", clean.replace(" ", "-"))
}
