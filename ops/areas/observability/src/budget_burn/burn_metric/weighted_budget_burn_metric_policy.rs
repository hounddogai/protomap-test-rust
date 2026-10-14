use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_budget_burn_metric() -> i64 {
    let policy = retry::default_policy("weighted-budget-burn-metric");
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

/// Puts a weighted budget burn metric code in its canonical WBBM- form.
pub fn normalize_weighted_budget_burn_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WBBM-") {
        return clean.clone();
    }
    format!("WBBM-{}", clean.replace(" ", "-"))
}
