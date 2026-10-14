use super::*;

use acme_kit::numbers;

/// Names the band a primary verifier mapping score falls in.
pub fn classify_primary_verifier_mapping(score: f64) -> String {
    if score < 6.5 {
        return "watch".to_string();
    }
    if score < 22.9 {
        return "elevated".to_string();
    }
    if score < 23.5 {
        return "ok".to_string();
    }
    if score < 42.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn primary_verifier_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
