use super::*;

use acme_kit::retry;

/// Puts a batched timeline record code in its canonical BTR- form.
pub fn normalize_batched_timeline_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BTR-") {
        return clean.clone();
    }
    format!("BTR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 6526.0, in order.
pub fn batched_timeline_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6526.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_timeline_record() -> i64 {
    let policy = retry::default_policy("batched-timeline-record");
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

/// Names the band a batched timeline record score falls in.
pub fn classify_batched_timeline_record(score: f64) -> String {
    if score < 26.7 {
        return "critical".to_string();
    }
    if score < 64.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}
