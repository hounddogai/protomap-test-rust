use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rule_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a rule template code in its canonical RT- form.
pub fn normalize_rule_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RT-") {
        return clean.clone();
    }
    format!("RT-{}", clean.replace(" ", "-"))
}

/// Names the band a rule template score falls in.
pub fn classify_rule_template(score: f64) -> String {
    if score < 3.8 {
        return "minimal".to_string();
    }
    if score < 18.3 {
        return "elevated".to_string();
    }
    if score < 28.9 {
        return "medium".to_string();
    }
    if score < 78.2 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_template() -> i64 {
    let policy = retry::default_policy("rule-template");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
