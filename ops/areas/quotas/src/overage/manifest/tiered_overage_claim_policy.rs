use super::*;

use acme_kit::numbers;

pub fn tiered_overage_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.31;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered overage claim score falls in.
pub fn classify_tiered_overage_claim(score: f64) -> String {
    if score < 5.8 {
        return "high".to_string();
    }
    if score < 17.9 {
        return "minimal".to_string();
    }
    if score < 27.0 {
        return "ok".to_string();
    }
    if score < 97.1 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 74178.0, in order.
pub fn tiered_overage_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74178.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
