use super::*;

use acme_kit::numbers;

/// Puts a global action item claim code in its canonical GAIC- form.
pub fn normalize_global_action_item_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GAIC-") {
        return clean.clone();
    }
    format!("GAIC-{}", clean.replace(" ", "-"))
}

pub fn global_action_item_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a global action item claim score falls in.
pub fn classify_global_action_item_claim(score: f64) -> String {
    if score < 11.3 {
        return "critical".to_string();
    }
    if score < 27.1 {
        return "minimal".to_string();
    }
    if score < 47.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 82885.0, in order.
pub fn global_action_item_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 82885.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
