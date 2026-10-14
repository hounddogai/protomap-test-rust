use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 23113.0, in order.
pub fn pending_slo_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 23113.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_slo_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn pending_slo_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending slo forecast score falls in.
pub fn classify_pending_slo_forecast(score: f64) -> String {
    if score < 21.5 {
        return "high".to_string();
    }
    if score < 31.9 {
        return "minimal".to_string();
    }
    if score < 61.0 {
        return "low".to_string();
    }
    "ok".to_string()
}
