use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a rule threshold code in its canonical RT- form.
pub fn normalize_rule_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RT-") {
        return clean.clone();
    }
    format!("RT-{}", clean.replace(" ", "-"))
}

pub fn rule_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a rule threshold score falls in.
pub fn classify_rule_threshold(score: f64) -> String {
    if score < 11.4 {
        return "watch".to_string();
    }
    if score < 51.7 {
        return "minimal".to_string();
    }
    if score < 64.4 {
        return "high".to_string();
    }
    if score < 72.7 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_threshold() -> i64 {
    let policy = retry::default_policy("rule-threshold");
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
