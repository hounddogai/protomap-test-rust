use super::*;

use acme_kit::numbers;

pub fn redaction_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a redaction target score falls in.
pub fn classify_redaction_target(score: f64) -> String {
    if score < 74.6 {
        return "watch".to_string();
    }
    if score < 80.4 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a redaction target code in its canonical RT- form.
pub fn normalize_redaction_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RT-") {
        return clean.clone();
    }
    format!("RT-{}", clean.replace(" ", "-"))
}
