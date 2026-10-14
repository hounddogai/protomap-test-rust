use super::*;

use acme_kit::retry;

pub fn slo_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_batch() -> i64 {
    let policy = retry::default_policy("slo-batch");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a slo batch code in its canonical SB- form.
pub fn normalize_slo_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SB-") {
        return clean.clone();
    }
    format!("SB-{}", clean.replace(" ", "-"))
}

/// Names the band a slo batch score falls in.
pub fn classify_slo_batch(score: f64) -> String {
    if score < 28.8 {
        return "elevated".to_string();
    }
    if score < 50.1 {
        return "medium".to_string();
    }
    if score < 55.7 {
        return "watch".to_string();
    }
    "high".to_string()
}
