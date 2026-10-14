use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_failover_cycle() -> i64 {
    let policy = retry::default_policy("legacy-failover-cycle");
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

/// Puts a legacy failover cycle code in its canonical LFC- form.
pub fn normalize_legacy_failover_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFC-") {
        return clean.clone();
    }
    format!("LFC-{}", clean.replace(" ", "-"))
}

pub fn legacy_failover_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
