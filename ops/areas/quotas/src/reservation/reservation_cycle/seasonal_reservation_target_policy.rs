use super::*;

use acme_kit::numbers;

/// Puts a seasonal reservation target code in its canonical SRT- form.
pub fn normalize_seasonal_reservation_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRT-") {
        return clean.clone();
    }
    format!("SRT-{}", clean.replace(" ", "-"))
}

/// Names the band a seasonal reservation target score falls in.
pub fn classify_seasonal_reservation_target(score: f64) -> String {
    if score < 44.4 {
        return "low".to_string();
    }
    if score < 47.5 {
        return "elevated".to_string();
    }
    if score < 77.5 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn seasonal_reservation_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 43663.0, in order.
pub fn seasonal_reservation_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43663.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
