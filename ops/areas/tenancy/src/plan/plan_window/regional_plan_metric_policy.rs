use super::*;

use acme_kit::retry;

/// Puts a regional plan metric code in its canonical RPM- form.
pub fn normalize_regional_plan_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPM-") {
        return clean.clone();
    }
    format!("RPM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 53186.0, in order.
pub fn regional_plan_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53186.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_plan_metric() -> i64 {
    let policy = retry::default_policy("regional-plan-metric");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
