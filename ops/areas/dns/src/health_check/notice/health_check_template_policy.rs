use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_health_check_template() -> i64 {
    let policy = retry::default_policy("health-check-template");
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

/// Counts how many amounts fit in a budget of 6752.0, in order.
pub fn health_check_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6752.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn health_check_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Puts a health check template code in its canonical HCT- form.
pub fn normalize_health_check_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCT-") {
        return clean.clone();
    }
    format!("HCT-{}", clean.replace(" ", "-"))
}
