use super::*;

use acme_kit::retry;

pub fn canary_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a canary ledger score falls in.
pub fn classify_canary_ledger(score: f64) -> String {
    if score < 9.1 {
        return "elevated".to_string();
    }
    if score < 74.1 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a canary ledger code in its canonical CL- form.
pub fn normalize_canary_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CL-") {
        return clean.clone();
    }
    format!("CL-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_canary_ledger() -> i64 {
    let policy = retry::default_policy("canary-ledger");
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
