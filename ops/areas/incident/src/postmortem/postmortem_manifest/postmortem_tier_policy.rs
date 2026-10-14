use super::*;

use acme_kit::numbers;

pub fn postmortem_tier_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn postmortem_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
