use super::*;

use acme_kit::numbers;

/// Names the band a federation mapping score falls in.
pub fn classify_federation_mapping(score: f64) -> String {
    if score < 21.3 {
        return "high".to_string();
    }
    if score < 36.9 {
        return "critical".to_string();
    }
    if score < 50.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn federation_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
