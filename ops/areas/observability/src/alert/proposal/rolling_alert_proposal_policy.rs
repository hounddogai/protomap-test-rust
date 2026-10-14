use super::*;

use acme_kit::numbers;

/// Puts a rolling alert proposal code in its canonical RAP- form.
pub fn normalize_rolling_alert_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RAP-") {
        return clean.clone();
    }
    format!("RAP-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling alert proposal score falls in.
pub fn classify_rolling_alert_proposal(score: f64) -> String {
    if score < 45.6 {
        return "high".to_string();
    }
    if score < 61.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn rolling_alert_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.05;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
