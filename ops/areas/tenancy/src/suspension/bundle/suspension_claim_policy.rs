use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn suspension_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a suspension claim score falls in.
pub fn classify_suspension_claim(score: f64) -> String {
    if score < 36.1 {
        return "low".to_string();
    }
    if score < 80.8 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Puts a suspension claim code in its canonical SC- form.
pub fn normalize_suspension_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SC-") {
        return clean.clone();
    }
    format!("SC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_claim() -> i64 {
    let policy = retry::default_policy("suspension-claim");
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
