use super::*;

use acme_kit::numbers;

pub fn reservation_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a reservation mapping score falls in.
pub fn classify_reservation_mapping(score: f64) -> String {
    if score < 20.4 {
        return "low".to_string();
    }
    if score < 22.9 {
        return "ok".to_string();
    }
    if score < 44.3 {
        return "high".to_string();
    }
    if score < 63.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
