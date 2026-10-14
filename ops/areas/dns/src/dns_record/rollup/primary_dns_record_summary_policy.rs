use super::*;

use acme_kit::numbers;

pub fn primary_dns_record_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn primary_dns_record_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a primary dns record summary score falls in.
pub fn classify_primary_dns_record_summary(score: f64) -> String {
    if score < 15.0 {
        return "watch".to_string();
    }
    if score < 26.7 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 30579.0, in order.
pub fn primary_dns_record_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30579.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
