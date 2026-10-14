use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 95546.0, in order.
pub fn legacy_failover_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95546.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_failover_digest() -> i64 {
    let policy = retry::default_policy("legacy-failover-digest");
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

/// Puts a legacy failover digest code in its canonical LFD- form.
pub fn normalize_legacy_failover_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFD-") {
        return clean.clone();
    }
    format!("LFD-{}", clean.replace(" ", "-"))
}

pub fn legacy_failover_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
