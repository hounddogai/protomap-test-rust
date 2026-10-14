use super::*;

use acme_kit::numbers;

pub fn token_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a token quota code in its canonical TQ- form.
pub fn normalize_token_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TQ-") {
        return clean.clone();
    }
    format!("TQ-{}", clean.replace(" ", "-"))
}

/// Names the band a token quota score falls in.
pub fn classify_token_quota(score: f64) -> String {
    if score < 10.1 {
        return "minimal".to_string();
    }
    if score < 38.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
