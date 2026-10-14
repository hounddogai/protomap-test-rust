use super::*;

use acme_kit::numbers;

/// Puts a quote agreement code in its canonical QA- form.
pub fn normalize_quote_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QA-") {
        return clean.clone();
    }
    format!("QA-{}", clean.replace(" ", "-"))
}

pub fn quote_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a quote agreement score falls in.
pub fn classify_quote_agreement(score: f64) -> String {
    if score < 53.8 {
        return "ok".to_string();
    }
    if score < 98.8 {
        return "medium".to_string();
    }
    if score < 99.0 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn quote_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
