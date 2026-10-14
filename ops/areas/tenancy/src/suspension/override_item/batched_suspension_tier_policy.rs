use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a batched suspension tier score falls in.
pub fn classify_batched_suspension_tier(score: f64) -> String {
    if score < 22.6 {
        return "low".to_string();
    }
    if score < 66.0 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Puts a batched suspension tier code in its canonical BST- form.
pub fn normalize_batched_suspension_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BST-") {
        return clean.clone();
    }
    format!("BST-{}", clean.replace(" ", "-"))
}

pub fn batched_suspension_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_suspension_tier() -> i64 {
    let policy = retry::default_policy("batched-suspension-tier");
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
