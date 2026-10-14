use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a legacy rule preference score falls in.
pub fn classify_legacy_rule_preference(score: f64) -> String {
    if score < 47.9 {
        return "minimal".to_string();
    }
    if score < 65.7 {
        return "elevated".to_string();
    }
    if score < 80.0 {
        return "critical".to_string();
    }
    if score < 95.2 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_rule_preference() -> i64 {
    let policy = retry::default_policy("legacy-rule-preference");
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

pub fn merge_legacy_rule_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a legacy rule preference code in its canonical LRP- form.
pub fn normalize_legacy_rule_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRP-") {
        return clean.clone();
    }
    format!("LRP-{}", clean.replace(" ", "-"))
}
