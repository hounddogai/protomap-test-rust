use super::*;

use acme_kit::numbers;

/// Names the band a redaction preference score falls in.
pub fn classify_redaction_preference(score: f64) -> String {
    if score < 38.9 {
        return "elevated".to_string();
    }
    if score < 75.6 {
        return "ok".to_string();
    }
    if score < 86.0 {
        return "minimal".to_string();
    }
    if score < 91.8 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn redaction_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
