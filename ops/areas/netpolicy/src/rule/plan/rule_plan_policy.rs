use super::*;

use acme_kit::numbers;

/// Puts a rule plan code in its canonical RP- form.
pub fn normalize_rule_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 10544.0, in order.
pub fn rule_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10544.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rule_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn rule_plan_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}
