use super::*;

use acme_kit::numbers;

pub fn seasonal_quote_manifest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a seasonal quote manifest score falls in.
pub fn classify_seasonal_quote_manifest(score: f64) -> String {
    if score < 26.2 {
        return "minimal".to_string();
    }
    if score < 50.6 {
        return "ok".to_string();
    }
    if score < 55.2 {
        return "elevated".to_string();
    }
    if score < 97.5 {
        return "critical".to_string();
    }
    "low".to_string()
}
