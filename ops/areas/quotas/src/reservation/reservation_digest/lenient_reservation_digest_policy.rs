use super::*;

use acme_kit::numbers;

pub fn lenient_reservation_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a lenient reservation digest score falls in.
pub fn classify_lenient_reservation_digest(score: f64) -> String {
    if score < 43.6 {
        return "medium".to_string();
    }
    if score < 44.7 {
        return "watch".to_string();
    }
    if score < 80.4 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn lenient_reservation_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
