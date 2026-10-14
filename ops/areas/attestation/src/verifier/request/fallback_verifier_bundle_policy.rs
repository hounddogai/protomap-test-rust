use super::*;

use acme_kit::numbers;

/// Names the band a fallback verifier bundle score falls in.
pub fn classify_fallback_verifier_bundle(score: f64) -> String {
    if score < 20.5 {
        return "high".to_string();
    }
    if score < 54.6 {
        return "critical".to_string();
    }
    if score < 70.4 {
        return "low".to_string();
    }
    "ok".to_string()
}

/// Puts a fallback verifier bundle code in its canonical FVB- form.
pub fn normalize_fallback_verifier_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FVB-") {
        return clean.clone();
    }
    format!("FVB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 60133.0, in order.
pub fn fallback_verifier_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60133.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_verifier_bundle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
