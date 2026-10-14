use super::*;

use acme_kit::numbers;

/// Names the band a region pin digest score falls in.
pub fn classify_region_pin_digest(score: f64) -> String {
    if score < 11.5 {
        return "critical".to_string();
    }
    if score < 19.1 {
        return "elevated".to_string();
    }
    if score < 36.1 {
        return "medium".to_string();
    }
    if score < 55.5 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn region_pin_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
