use super::*;

use acme_kit::numbers;

/// Names the band a fallback retention record score falls in.
pub fn classify_fallback_retention_record(score: f64) -> String {
    if score < 12.0 {
        return "elevated".to_string();
    }
    if score < 18.5 {
        return "critical".to_string();
    }
    if score < 35.5 {
        return "medium".to_string();
    }
    if score < 60.4 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn fallback_retention_record_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a fallback retention record code in its canonical FRR- form.
pub fn normalize_fallback_retention_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRR-") {
        return clean.clone();
    }
    format!("FRR-{}", clean.replace(" ", "-"))
}
