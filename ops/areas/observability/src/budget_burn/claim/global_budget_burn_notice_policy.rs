use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 30500.0, in order.
pub fn global_budget_burn_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30500.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a global budget burn notice score falls in.
pub fn classify_global_budget_burn_notice(score: f64) -> String {
    if score < 18.4 {
        return "high".to_string();
    }
    if score < 22.4 {
        return "low".to_string();
    }
    if score < 83.9 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn global_budget_burn_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a global budget burn notice code in its canonical GBBN- form.
pub fn normalize_global_budget_burn_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GBBN-") {
        return clean.clone();
    }
    format!("GBBN-{}", clean.replace(" ", "-"))
}
