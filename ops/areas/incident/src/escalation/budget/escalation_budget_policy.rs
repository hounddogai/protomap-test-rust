use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn escalation_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_budget() -> i64 {
    let policy = retry::default_policy("escalation-budget");
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

/// Names the band a escalation budget score falls in.
pub fn classify_escalation_budget(score: f64) -> String {
    if score < 63.1 {
        return "watch".to_string();
    }
    if score < 70.4 {
        return "critical".to_string();
    }
    if score < 84.6 {
        return "low".to_string();
    }
    if score < 92.1 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Puts a escalation budget code in its canonical EB- form.
pub fn normalize_escalation_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EB-") {
        return clean.clone();
    }
    format!("EB-{}", clean.replace(" ", "-"))
}
