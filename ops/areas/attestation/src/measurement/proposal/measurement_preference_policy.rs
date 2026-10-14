use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a measurement preference code in its canonical MP- form.
pub fn normalize_measurement_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MP-") {
        return clean.clone();
    }
    format!("MP-{}", clean.replace(" ", "-"))
}

/// Names the band a measurement preference score falls in.
pub fn classify_measurement_preference(score: f64) -> String {
    if score < 3.0 {
        return "high".to_string();
    }
    if score < 18.5 {
        return "elevated".to_string();
    }
    if score < 42.7 {
        return "low".to_string();
    }
    if score < 54.3 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn merge_measurement_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_preference() -> i64 {
    let policy = retry::default_policy("measurement-preference");
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
