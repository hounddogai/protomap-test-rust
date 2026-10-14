use super::*;

use acme_kit::retry;

/// Puts a retention claim code in its canonical RC- form.
pub fn normalize_retention_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_retention_claim() -> i64 {
    let policy = retry::default_policy("retention-claim");
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

/// Names the band a retention claim score falls in.
pub fn classify_retention_claim(score: f64) -> String {
    if score < 6.8 {
        return "watch".to_string();
    }
    if score < 68.2 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn retention_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}
