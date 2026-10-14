use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a lenient redaction forecast score falls in.
pub fn classify_lenient_redaction_forecast(score: f64) -> String {
    if score < 24.2 {
        return "medium".to_string();
    }
    if score < 72.5 {
        return "minimal".to_string();
    }
    if score < 80.0 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn merge_lenient_redaction_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
