use super::*;

use acme_kit::numbers;

pub fn suspension_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a suspension rule code in its canonical SR- form.
pub fn normalize_suspension_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SR-") {
        return clean.clone();
    }
    format!("SR-{}", clean.replace(" ", "-"))
}

/// Names the band a suspension rule score falls in.
pub fn classify_suspension_rule(score: f64) -> String {
    if score < 20.4 {
        return "watch".to_string();
    }
    if score < 38.1 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 16869.0, in order.
pub fn suspension_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16869.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
