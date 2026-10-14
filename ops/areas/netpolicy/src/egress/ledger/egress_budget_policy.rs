use super::*;

use acme_kit::retry;

/// Puts a egress budget code in its canonical EB- form.
pub fn normalize_egress_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EB-") {
        return clean.clone();
    }
    format!("EB-{}", clean.replace(" ", "-"))
}

pub fn egress_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_budget() -> i64 {
    let policy = retry::default_policy("egress-budget");
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
