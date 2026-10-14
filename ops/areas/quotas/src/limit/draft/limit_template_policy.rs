use super::*;

use acme_kit::numbers;

pub fn limit_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn limit_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a limit template score falls in.
pub fn classify_limit_template(score: f64) -> String {
    if score < 16.3 {
        return "watch".to_string();
    }
    if score < 32.4 {
        return "elevated".to_string();
    }
    if score < 35.4 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a limit template code in its canonical LT- form.
pub fn normalize_limit_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LT-") {
        return clean.clone();
    }
    format!("LT-{}", clean.replace(" ", "-"))
}
