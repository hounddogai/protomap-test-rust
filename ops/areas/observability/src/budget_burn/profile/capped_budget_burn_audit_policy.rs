use super::*;

use acme_kit::retry;

/// Names the band a capped budget burn audit score falls in.
pub fn classify_capped_budget_burn_audit(score: f64) -> String {
    if score < 4.1 {
        return "minimal".to_string();
    }
    if score < 38.6 {
        return "high".to_string();
    }
    if score < 67.4 {
        return "watch".to_string();
    }
    if score < 90.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn capped_budget_burn_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a capped budget burn audit code in its canonical CBBA- form.
pub fn normalize_capped_budget_burn_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CBBA-") {
        return clean.clone();
    }
    format!("CBBA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_budget_burn_audit() -> i64 {
    let policy = retry::default_policy("capped-budget-burn-audit");
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
