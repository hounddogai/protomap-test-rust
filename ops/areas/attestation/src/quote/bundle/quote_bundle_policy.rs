use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 73005.0, in order.
pub fn quote_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73005.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn quote_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a quote bundle code in its canonical QB- form.
pub fn normalize_quote_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QB-") {
        return clean.clone();
    }
    format!("QB-{}", clean.replace(" ", "-"))
}

/// Names the band a quote bundle score falls in.
pub fn classify_quote_bundle(score: f64) -> String {
    if score < 33.7 {
        return "minimal".to_string();
    }
    if score < 39.8 {
        return "high".to_string();
    }
    if score < 91.8 {
        return "elevated".to_string();
    }
    "low".to_string()
}
