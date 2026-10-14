use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 38117.0, in order.
pub fn lenient_forecast_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38117.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_forecast_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a lenient forecast estimate score falls in.
pub fn classify_lenient_forecast_estimate(score: f64) -> String {
    if score < 17.1 {
        return "elevated".to_string();
    }
    if score < 18.6 {
        return "ok".to_string();
    }
    if score < 74.8 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn lenient_forecast_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
