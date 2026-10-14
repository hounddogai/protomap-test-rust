use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a adaptive retention override score falls in.
pub fn classify_adaptive_retention_override(score: f64) -> String {
    if score < 13.3 {
        return "ok".to_string();
    }
    if score < 52.4 {
        return "elevated".to_string();
    }
    if score < 90.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn adaptive_retention_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a adaptive retention override code in its canonical ARO- form.
pub fn normalize_adaptive_retention_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARO-") {
        return clean.clone();
    }
    format!("ARO-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_retention_override() -> i64 {
    let policy = retry::default_policy("adaptive-retention-override");
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
