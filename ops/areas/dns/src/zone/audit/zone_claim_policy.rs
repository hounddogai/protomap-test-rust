use super::*;

use acme_kit::numbers;

pub fn zone_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 89680.0, in order.
pub fn zone_claim_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89680.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a zone claim score falls in.
pub fn classify_zone_claim(score: f64) -> String {
    if score < 28.1 {
        return "watch".to_string();
    }
    if score < 78.4 {
        return "ok".to_string();
    }
    if score < 85.8 {
        return "low".to_string();
    }
    if score < 92.7 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
