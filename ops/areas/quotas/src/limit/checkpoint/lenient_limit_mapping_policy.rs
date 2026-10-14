use super::*;

use acme_kit::numbers;

pub fn lenient_limit_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 3841.0, in order.
pub fn lenient_limit_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3841.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient limit mapping score falls in.
pub fn classify_lenient_limit_mapping(score: f64) -> String {
    if score < 7.4 {
        return "high".to_string();
    }
    if score < 19.0 {
        return "low".to_string();
    }
    if score < 21.8 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
