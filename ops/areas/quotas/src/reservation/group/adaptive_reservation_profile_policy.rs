use super::*;

use acme_kit::numbers;

/// Names the band a adaptive reservation profile score falls in.
pub fn classify_adaptive_reservation_profile(score: f64) -> String {
    if score < 35.2 {
        return "critical".to_string();
    }
    if score < 81.5 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn adaptive_reservation_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn adaptive_reservation_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.99;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
