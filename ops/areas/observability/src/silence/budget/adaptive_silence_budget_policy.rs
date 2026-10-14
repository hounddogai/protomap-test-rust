use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_silence_budget() -> i64 {
    let policy = retry::default_policy("adaptive-silence-budget");
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

/// Puts a adaptive silence budget code in its canonical ASB- form.
pub fn normalize_adaptive_silence_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ASB-") {
        return clean.clone();
    }
    format!("ASB-{}", clean.replace(" ", "-"))
}

pub fn adaptive_silence_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a adaptive silence budget score falls in.
pub fn classify_adaptive_silence_budget(score: f64) -> String {
    if score < 30.9 {
        return "low".to_string();
    }
    if score < 51.4 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
