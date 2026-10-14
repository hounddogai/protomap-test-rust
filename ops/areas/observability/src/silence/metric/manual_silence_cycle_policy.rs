use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_manual_silence_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a manual silence cycle code in its canonical MSC- form.
pub fn normalize_manual_silence_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSC-") {
        return clean.clone();
    }
    format!("MSC-{}", clean.replace(" ", "-"))
}

/// Names the band a manual silence cycle score falls in.
pub fn classify_manual_silence_cycle(score: f64) -> String {
    if score < 49.2 {
        return "critical".to_string();
    }
    if score < 49.5 {
        return "watch".to_string();
    }
    if score < 70.2 {
        return "elevated".to_string();
    }
    if score < 80.2 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_silence_cycle() -> i64 {
    let policy = retry::default_policy("manual-silence-cycle");
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
