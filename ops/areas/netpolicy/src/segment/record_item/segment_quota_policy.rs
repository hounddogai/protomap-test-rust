use super::*;

use acme_kit::numbers;

pub fn segment_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn segment_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 25125.0, in order.
pub fn segment_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25125.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a segment quota score falls in.
pub fn classify_segment_quota(score: f64) -> String {
    if score < 34.2 {
        return "critical".to_string();
    }
    if score < 46.8 {
        return "minimal".to_string();
    }
    "high".to_string()
}
