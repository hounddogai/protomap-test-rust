use super::*;

use acme_kit::retry;

/// Puts a runbook notice code in its canonical RN- form.
pub fn normalize_runbook_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RN-") {
        return clean.clone();
    }
    format!("RN-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_notice() -> i64 {
    let policy = retry::default_policy("runbook-notice");
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

/// Counts how many amounts fit in a budget of 29710.0, in order.
pub fn runbook_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 29710.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn runbook_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
