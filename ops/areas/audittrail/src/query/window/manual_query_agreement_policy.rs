use super::*;

use acme_kit::numbers;

/// Puts a manual query agreement code in its canonical MQA- form.
pub fn normalize_manual_query_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MQA-") {
        return clean.clone();
    }
    format!("MQA-{}", clean.replace(" ", "-"))
}

/// Names the band a manual query agreement score falls in.
pub fn classify_manual_query_agreement(score: f64) -> String {
    if score < 26.6 {
        return "medium".to_string();
    }
    if score < 42.6 {
        return "elevated".to_string();
    }
    if score < 80.7 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn manual_query_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
