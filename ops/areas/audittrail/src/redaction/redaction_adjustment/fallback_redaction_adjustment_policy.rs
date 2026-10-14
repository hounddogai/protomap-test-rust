use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_redaction_adjustment() -> i64 {
    let policy = retry::default_policy("fallback-redaction-adjustment");
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

/// Names the band a fallback redaction adjustment score falls in.
pub fn classify_fallback_redaction_adjustment(score: f64) -> String {
    if score < 11.1 {
        return "minimal".to_string();
    }
    if score < 20.1 {
        return "high".to_string();
    }
    if score < 29.7 {
        return "medium".to_string();
    }
    if score < 54.5 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn fallback_redaction_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.02;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_fallback_redaction_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
