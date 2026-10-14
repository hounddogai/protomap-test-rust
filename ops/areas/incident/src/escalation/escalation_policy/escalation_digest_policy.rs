use super::*;

use acme_kit::numbers;

pub fn escalation_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.17;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a escalation digest code in its canonical ED- form.
pub fn normalize_escalation_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ED-") {
        return clean.clone();
    }
    format!("ED-{}", clean.replace(" ", "-"))
}

/// Names the band a escalation digest score falls in.
pub fn classify_escalation_digest(score: f64) -> String {
    if score < 10.0 {
        return "high".to_string();
    }
    if score < 10.4 {
        return "low".to_string();
    }
    if score < 33.7 {
        return "elevated".to_string();
    }
    if score < 69.9 {
        return "ok".to_string();
    }
    "critical".to_string()
}
