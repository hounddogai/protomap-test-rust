use super::*;

use acme_kit::numbers;

pub fn pending_release_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.81;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending release digest score falls in.
pub fn classify_pending_release_digest(score: f64) -> String {
    if score < 35.8 {
        return "medium".to_string();
    }
    if score < 42.1 {
        return "elevated".to_string();
    }
    if score < 48.9 {
        return "high".to_string();
    }
    if score < 59.3 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn pending_release_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 38372.0, in order.
pub fn pending_release_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38372.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
