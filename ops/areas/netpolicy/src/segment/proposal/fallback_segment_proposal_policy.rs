use super::*;

use acme_kit::numbers;

/// Puts a fallback segment proposal code in its canonical FSP- form.
pub fn normalize_fallback_segment_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FSP-") {
        return clean.clone();
    }
    format!("FSP-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback segment proposal score falls in.
pub fn classify_fallback_segment_proposal(score: f64) -> String {
    if score < 43.4 {
        return "low".to_string();
    }
    if score < 79.4 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn fallback_segment_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
