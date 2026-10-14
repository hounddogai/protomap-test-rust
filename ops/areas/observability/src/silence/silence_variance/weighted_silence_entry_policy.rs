use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a weighted silence entry score falls in.
pub fn classify_weighted_silence_entry(score: f64) -> String {
    if score < 23.0 {
        return "watch".to_string();
    }
    if score < 34.5 {
        return "high".to_string();
    }
    if score < 56.3 {
        return "medium".to_string();
    }
    if score < 83.7 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Puts a weighted silence entry code in its canonical WSE- form.
pub fn normalize_weighted_silence_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WSE-") {
        return clean.clone();
    }
    format!("WSE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_silence_entry() -> i64 {
    let policy = retry::default_policy("weighted-silence-entry");
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

pub fn merge_weighted_silence_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
