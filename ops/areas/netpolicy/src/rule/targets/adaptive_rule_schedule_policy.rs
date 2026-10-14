use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn adaptive_rule_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_rule_schedule() -> i64 {
    let policy = retry::default_policy("adaptive-rule-schedule");
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

/// Puts a adaptive rule schedule code in its canonical ARS- form.
pub fn normalize_adaptive_rule_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARS-") {
        return clean.clone();
    }
    format!("ARS-{}", clean.replace(" ", "-"))
}
