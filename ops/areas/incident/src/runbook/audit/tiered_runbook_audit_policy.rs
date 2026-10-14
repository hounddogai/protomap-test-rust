use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 39037.0, in order.
pub fn tiered_runbook_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39037.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tiered runbook audit score falls in.
pub fn classify_tiered_runbook_audit(score: f64) -> String {
    if score < 6.1 {
        return "minimal".to_string();
    }
    if score < 34.8 {
        return "critical".to_string();
    }
    if score < 44.5 {
        return "watch".to_string();
    }
    if score < 66.7 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_runbook_audit() -> i64 {
    let policy = retry::default_policy("tiered-runbook-audit");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a tiered runbook audit code in its canonical TRA- form.
pub fn normalize_tiered_runbook_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRA-") {
        return clean.clone();
    }
    format!("TRA-{}", clean.replace(" ", "-"))
}
