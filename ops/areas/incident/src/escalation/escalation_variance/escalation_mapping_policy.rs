use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a escalation mapping score falls in.
pub fn classify_escalation_mapping(score: f64) -> String {
    if score < 16.1 {
        return "critical".to_string();
    }
    if score < 35.9 {
        return "ok".to_string();
    }
    if score < 50.3 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Puts a escalation mapping code in its canonical EM- form.
pub fn normalize_escalation_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EM-") {
        return clean.clone();
    }
    format!("EM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_escalation_mapping() -> i64 {
    let policy = retry::default_policy("escalation-mapping");
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

pub fn merge_escalation_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
