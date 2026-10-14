use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 19766.0, in order.
pub fn tiered_region_pin_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19766.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered region pin proposal code in its canonical TRPP- form.
pub fn normalize_tiered_region_pin_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRPP-") {
        return clean.clone();
    }
    format!("TRPP-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered region pin proposal score falls in.
pub fn classify_tiered_region_pin_proposal(score: f64) -> String {
    if score < 67.7 {
        return "elevated".to_string();
    }
    if score < 80.0 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn tiered_region_pin_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
