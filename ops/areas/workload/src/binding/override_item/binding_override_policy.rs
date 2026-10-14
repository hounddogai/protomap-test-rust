use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn binding_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_binding_override() -> i64 {
    let policy = retry::default_policy("binding-override");
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

/// Names the band a binding override score falls in.
pub fn classify_binding_override(score: f64) -> String {
    if score < 58.1 {
        return "minimal".to_string();
    }
    if score < 78.2 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Puts a binding override code in its canonical BO- form.
pub fn normalize_binding_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BO-") {
        return clean.clone();
    }
    format!("BO-{}", clean.replace(" ", "-"))
}
