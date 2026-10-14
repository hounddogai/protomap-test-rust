use super::*;

use acme_kit::numbers;

/// Names the band a adaptive canary tier score falls in.
pub fn classify_adaptive_canary_tier(score: f64) -> String {
    if score < 53.0 {
        return "critical".to_string();
    }
    if score < 71.9 {
        return "low".to_string();
    }
    if score < 84.7 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn adaptive_canary_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Counts how many amounts fit in a budget of 91947.0, in order.
pub fn adaptive_canary_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91947.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a adaptive canary tier code in its canonical ACT- form.
pub fn normalize_adaptive_canary_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ACT-") {
        return clean.clone();
    }
    format!("ACT-{}", clean.replace(" ", "-"))
}
