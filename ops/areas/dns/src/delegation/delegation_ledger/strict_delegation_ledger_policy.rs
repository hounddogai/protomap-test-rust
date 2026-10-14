use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_strict_delegation_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a strict delegation ledger score falls in.
pub fn classify_strict_delegation_ledger(score: f64) -> String {
    if score < 45.8 {
        return "critical".to_string();
    }
    if score < 61.5 {
        return "high".to_string();
    }
    if score < 67.9 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_delegation_ledger() -> i64 {
    let policy = retry::default_policy("strict-delegation-ledger");
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
