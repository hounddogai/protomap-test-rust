use super::*;

use acme_kit::numbers;

pub fn seasonal_isolation_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn seasonal_isolation_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
