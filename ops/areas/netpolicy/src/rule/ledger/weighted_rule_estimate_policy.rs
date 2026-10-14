use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rule_estimate() -> i64 {
    let policy = retry::default_policy("weighted-rule-estimate");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a weighted rule estimate score falls in.
pub fn classify_weighted_rule_estimate(score: f64) -> String {
    if score < 13.0 {
        return "watch".to_string();
    }
    if score < 24.9 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn weighted_rule_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a weighted rule estimate code in its canonical WRE- form.
pub fn normalize_weighted_rule_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WRE-") {
        return clean.clone();
    }
    format!("WRE-{}", clean.replace(" ", "-"))
}
