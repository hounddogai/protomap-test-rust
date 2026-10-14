use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a scheduled evidence template score falls in.
pub fn classify_scheduled_evidence_template(score: f64) -> String {
    if score < 83.0 {
        return "ok".to_string();
    }
    if score < 95.2 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a scheduled evidence template code in its canonical SET- form.
pub fn normalize_scheduled_evidence_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SET-") {
        return clean.clone();
    }
    format!("SET-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_evidence_template() -> i64 {
    let policy = retry::default_policy("scheduled-evidence-template");
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

pub fn merge_scheduled_evidence_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
