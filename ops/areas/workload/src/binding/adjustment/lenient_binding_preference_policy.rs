use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 36030.0, in order.
pub fn lenient_binding_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36030.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a lenient binding preference code in its canonical LBP- form.
pub fn normalize_lenient_binding_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LBP-") {
        return clean.clone();
    }
    format!("LBP-{}", clean.replace(" ", "-"))
}

pub fn lenient_binding_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a lenient binding preference score falls in.
pub fn classify_lenient_binding_preference(score: f64) -> String {
    if score < 38.9 {
        return "watch".to_string();
    }
    if score < 77.2 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
