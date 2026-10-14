use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_sink_ledger() -> i64 {
    let policy = retry::default_policy("legacy-sink-ledger");
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

pub fn merge_legacy_sink_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a legacy sink ledger code in its canonical LSL- form.
pub fn normalize_legacy_sink_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LSL-") {
        return clean.clone();
    }
    format!("LSL-{}", clean.replace(" ", "-"))
}

pub fn legacy_sink_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
