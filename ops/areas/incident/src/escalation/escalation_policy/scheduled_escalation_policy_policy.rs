use super::*;

use acme_kit::numbers;

/// Puts a scheduled escalation policy code in its canonical SEP- form.
pub fn normalize_scheduled_escalation_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEP-") {
        return clean.clone();
    }
    format!("SEP-{}", clean.replace(" ", "-"))
}

/// Names the band a scheduled escalation policy score falls in.
pub fn classify_scheduled_escalation_policy(score: f64) -> String {
    if score < 42.6 {
        return "watch".to_string();
    }
    if score < 70.2 {
        return "low".to_string();
    }
    if score < 80.4 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn scheduled_escalation_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
