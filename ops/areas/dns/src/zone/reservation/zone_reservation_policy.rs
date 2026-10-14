use super::*;

use acme_kit::numbers;

/// Names the band a zone reservation score falls in.
pub fn classify_zone_reservation(score: f64) -> String {
    if score < 4.8 {
        return "medium".to_string();
    }
    if score < 37.5 {
        return "ok".to_string();
    }
    if score < 64.6 {
        return "minimal".to_string();
    }
    if score < 91.8 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn zone_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
