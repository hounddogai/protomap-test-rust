use super::*;

use acme_kit::numbers;

/// Names the band a canary request score falls in.
pub fn classify_canary_request(score: f64) -> String {
    if score < 1.7 {
        return "minimal".to_string();
    }
    if score < 72.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn canary_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a canary request code in its canonical CR- form.
pub fn normalize_canary_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CR-") {
        return clean.clone();
    }
    format!("CR-{}", clean.replace(" ", "-"))
}
