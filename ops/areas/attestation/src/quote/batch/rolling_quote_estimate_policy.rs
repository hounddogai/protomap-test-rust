use super::*;

use acme_kit::numbers;

pub fn rolling_quote_estimate_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a rolling quote estimate score falls in.
pub fn classify_rolling_quote_estimate(score: f64) -> String {
    if score < 14.3 {
        return "high".to_string();
    }
    if score < 30.5 {
        return "elevated".to_string();
    }
    if score < 49.1 {
        return "medium".to_string();
    }
    if score < 54.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Puts a rolling quote estimate code in its canonical RQE- form.
pub fn normalize_rolling_quote_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RQE-") {
        return clean.clone();
    }
    format!("RQE-{}", clean.replace(" ", "-"))
}
