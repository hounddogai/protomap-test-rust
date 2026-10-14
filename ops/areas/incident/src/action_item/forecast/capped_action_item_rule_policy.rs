use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a capped action item rule code in its canonical CAIR- form.
pub fn normalize_capped_action_item_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CAIR-") {
        return clean.clone();
    }
    format!("CAIR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_action_item_rule() -> i64 {
    let policy = retry::default_policy("capped-action-item-rule");
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

pub fn capped_action_item_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a capped action item rule score falls in.
pub fn classify_capped_action_item_rule(score: f64) -> String {
    if score < 5.1 {
        return "minimal".to_string();
    }
    if score < 6.3 {
        return "low".to_string();
    }
    if score < 61.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}
