use super::*;

use acme_kit::numbers;

/// Names the band a strict query reservation score falls in.
pub fn classify_strict_query_reservation(score: f64) -> String {
    if score < 37.5 {
        return "low".to_string();
    }
    if score < 60.7 {
        return "watch".to_string();
    }
    if score < 84.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn strict_query_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn strict_query_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
