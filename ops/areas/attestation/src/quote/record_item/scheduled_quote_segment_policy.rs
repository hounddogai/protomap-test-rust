use super::*;

use acme_kit::numbers;

/// Names the band a scheduled quote segment score falls in.
pub fn classify_scheduled_quote_segment(score: f64) -> String {
    if score < 10.6 {
        return "high".to_string();
    }
    if score < 75.3 {
        return "elevated".to_string();
    }
    if score < 93.9 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Puts a scheduled quote segment code in its canonical SQS- form.
pub fn normalize_scheduled_quote_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQS-") {
        return clean.clone();
    }
    format!("SQS-{}", clean.replace(" ", "-"))
}

pub fn scheduled_quote_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
