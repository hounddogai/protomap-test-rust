use super::*;

use acme_kit::numbers;

pub fn trust_domain_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a trust domain slot score falls in.
pub fn classify_trust_domain_slot(score: f64) -> String {
    if score < 1.1 {
        return "watch".to_string();
    }
    if score < 2.9 {
        return "ok".to_string();
    }
    if score < 45.9 {
        return "minimal".to_string();
    }
    if score < 55.9 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 35608.0, in order.
pub fn trust_domain_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35608.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
