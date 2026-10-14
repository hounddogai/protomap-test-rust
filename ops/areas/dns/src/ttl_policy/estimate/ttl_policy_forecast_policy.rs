use super::*;

use acme_kit::numbers;

pub fn ttl_policy_forecast_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 53009.0, in order.
pub fn ttl_policy_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53009.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a ttl policy forecast score falls in.
pub fn classify_ttl_policy_forecast(score: f64) -> String {
    if score < 12.6 {
        return "critical".to_string();
    }
    if score < 32.2 {
        return "elevated".to_string();
    }
    if score < 45.8 {
        return "watch".to_string();
    }
    if score < 62.0 {
        return "ok".to_string();
    }
    "high".to_string()
}
