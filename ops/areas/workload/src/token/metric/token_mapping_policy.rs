use super::*;

use acme_kit::numbers;

/// Names the band a token mapping score falls in.
pub fn classify_token_mapping(score: f64) -> String {
    if score < 70.3 {
        return "minimal".to_string();
    }
    if score < 70.3 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Puts a token mapping code in its canonical TM- form.
pub fn normalize_token_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TM-") {
        return clean.clone();
    }
    format!("TM-{}", clean.replace(" ", "-"))
}

pub fn token_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
