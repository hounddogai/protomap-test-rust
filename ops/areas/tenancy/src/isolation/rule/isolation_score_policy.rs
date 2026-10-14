use super::*;

use acme_kit::retry;

/// Puts a isolation score code in its canonical IS- form.
pub fn normalize_isolation_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IS-") {
        return clean.clone();
    }
    format!("IS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_isolation_score() -> i64 {
    let policy = retry::default_policy("isolation-score");
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

pub fn isolation_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a isolation score score falls in.
pub fn classify_isolation_score(score: f64) -> String {
    if score < 9.7 {
        return "low".to_string();
    }
    if score < 27.3 {
        return "medium".to_string();
    }
    if score < 38.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}
