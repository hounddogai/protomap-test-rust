use super::*;

use acme_kit::retry;

pub fn capped_peering_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a capped peering budget score falls in.
pub fn classify_capped_peering_budget(score: f64) -> String {
    if score < 3.5 {
        return "critical".to_string();
    }
    if score < 26.2 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_peering_budget() -> i64 {
    let policy = retry::default_policy("capped-peering-budget");
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

/// Puts a capped peering budget code in its canonical CPB- form.
pub fn normalize_capped_peering_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CPB-") {
        return clean.clone();
    }
    format!("CPB-{}", clean.replace(" ", "-"))
}
