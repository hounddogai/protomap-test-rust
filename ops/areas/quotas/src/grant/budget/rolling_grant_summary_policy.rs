use super::*;

use acme_kit::numbers;

/// Puts a rolling grant summary code in its canonical RGS- form.
pub fn normalize_rolling_grant_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RGS-") {
        return clean.clone();
    }
    format!("RGS-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling grant summary score falls in.
pub fn classify_rolling_grant_summary(score: f64) -> String {
    if score < 33.8 {
        return "critical".to_string();
    }
    if score < 61.0 {
        return "low".to_string();
    }
    if score < 99.1 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

pub fn rolling_grant_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
