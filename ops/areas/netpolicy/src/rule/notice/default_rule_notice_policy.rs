use super::*;

use acme_kit::retry;

/// Puts a default rule notice code in its canonical DRN- form.
pub fn normalize_default_rule_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRN-") {
        return clean.clone();
    }
    format!("DRN-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_rule_notice() -> i64 {
    let policy = retry::default_policy("default-rule-notice");
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

/// Counts how many amounts fit in a budget of 1930.0, in order.
pub fn default_rule_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1930.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
