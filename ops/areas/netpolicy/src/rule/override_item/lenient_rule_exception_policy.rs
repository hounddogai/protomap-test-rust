use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a lenient rule exception code in its canonical LRE- form.
pub fn normalize_lenient_rule_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRE-") {
        return clean.clone();
    }
    format!("LRE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_rule_exception() -> i64 {
    let policy = retry::default_policy("lenient-rule-exception");
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

pub fn lenient_rule_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
