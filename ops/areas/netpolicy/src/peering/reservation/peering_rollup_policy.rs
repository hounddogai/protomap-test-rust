use super::*;

use acme_kit::retry;

pub fn peering_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a peering rollup code in its canonical PR- form.
pub fn normalize_peering_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PR-") {
        return clean.clone();
    }
    format!("PR-{}", clean.replace(" ", "-"))
}

/// Names the band a peering rollup score falls in.
pub fn classify_peering_rollup(score: f64) -> String {
    if score < 22.9 {
        return "ok".to_string();
    }
    if score < 31.6 {
        return "low".to_string();
    }
    if score < 33.8 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_rollup() -> i64 {
    let policy = retry::default_policy("peering-rollup");
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
