use super::*;

use acme_kit::retry;

/// Names the band a timeline audit score falls in.
pub fn classify_timeline_audit(score: f64) -> String {
    if score < 14.0 {
        return "watch".to_string();
    }
    if score < 19.3 {
        return "critical".to_string();
    }
    if score < 80.2 {
        return "high".to_string();
    }
    if score < 85.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_audit() -> i64 {
    let policy = retry::default_policy("timeline-audit");
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

/// Counts how many amounts fit in a budget of 17215.0, in order.
pub fn timeline_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17215.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a timeline audit code in its canonical TA- form.
pub fn normalize_timeline_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TA-") {
        return clean.clone();
    }
    format!("TA-{}", clean.replace(" ", "-"))
}
