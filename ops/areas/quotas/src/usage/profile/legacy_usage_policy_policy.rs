use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a legacy usage policy score falls in.
pub fn classify_legacy_usage_policy(score: f64) -> String {
    if score < 11.1 {
        return "minimal".to_string();
    }
    if score < 11.2 {
        return "medium".to_string();
    }
    if score < 45.6 {
        return "ok".to_string();
    }
    if score < 89.5 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Puts a legacy usage policy code in its canonical LUP- form.
pub fn normalize_legacy_usage_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LUP-") {
        return clean.clone();
    }
    format!("LUP-{}", clean.replace(" ", "-"))
}

pub fn legacy_usage_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_usage_policy() -> i64 {
    let policy = retry::default_policy("legacy-usage-policy");
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
