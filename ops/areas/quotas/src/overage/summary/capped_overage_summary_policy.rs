use super::*;

use acme_kit::retry;

/// Puts a capped overage summary code in its canonical COS- form.
pub fn normalize_capped_overage_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("COS-") {
        return clean.clone();
    }
    format!("COS-{}", clean.replace(" ", "-"))
}

/// Names the band a capped overage summary score falls in.
pub fn classify_capped_overage_summary(score: f64) -> String {
    if score < 8.2 {
        return "elevated".to_string();
    }
    if score < 24.6 {
        return "critical".to_string();
    }
    if score < 92.0 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_overage_summary() -> i64 {
    let policy = retry::default_policy("capped-overage-summary");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn capped_overage_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
