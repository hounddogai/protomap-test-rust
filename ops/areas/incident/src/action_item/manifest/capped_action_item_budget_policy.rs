use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a capped action item budget code in its canonical CAIB- form.
pub fn normalize_capped_action_item_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CAIB-") {
        return clean.clone();
    }
    format!("CAIB-{}", clean.replace(" ", "-"))
}

/// Names the band a capped action item budget score falls in.
pub fn classify_capped_action_item_budget(score: f64) -> String {
    if score < 50.4 {
        return "critical".to_string();
    }
    if score < 62.3 {
        return "minimal".to_string();
    }
    if score < 69.0 {
        return "watch".to_string();
    }
    if score < 78.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

pub fn merge_capped_action_item_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_action_item_budget() -> i64 {
    let policy = retry::default_policy("capped-action-item-budget");
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
