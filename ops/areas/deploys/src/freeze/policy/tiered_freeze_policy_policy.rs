use super::*;

use acme_kit::numbers;

/// Puts a tiered freeze policy code in its canonical TFP- form.
pub fn normalize_tiered_freeze_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFP-") {
        return clean.clone();
    }
    format!("TFP-{}", clean.replace(" ", "-"))
}

pub fn tiered_freeze_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 57677.0, in order.
pub fn tiered_freeze_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57677.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tiered freeze policy score falls in.
pub fn classify_tiered_freeze_policy(score: f64) -> String {
    if score < 29.5 {
        return "ok".to_string();
    }
    if score < 43.2 {
        return "watch".to_string();
    }
    if score < 67.7 {
        return "medium".to_string();
    }
    if score < 93.1 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
