use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 5181.0, in order.
pub fn action_item_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5181.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a action item target code in its canonical AIT- form.
pub fn normalize_action_item_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIT-") {
        return clean.clone();
    }
    format!("AIT-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_target() -> i64 {
    let policy = retry::default_policy("action-item-target");
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

pub fn action_item_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
