use super::*;

use acme_kit::numbers;

/// Names the band a global reservation digest score falls in.
pub fn classify_global_reservation_digest(score: f64) -> String {
    if score < 3.2 {
        return "critical".to_string();
    }
    if score < 54.8 {
        return "watch".to_string();
    }
    if score < 85.1 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn global_reservation_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
