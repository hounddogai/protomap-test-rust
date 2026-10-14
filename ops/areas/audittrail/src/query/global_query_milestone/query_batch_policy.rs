use super::*;

use acme_kit::numbers;

pub fn query_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn query_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
