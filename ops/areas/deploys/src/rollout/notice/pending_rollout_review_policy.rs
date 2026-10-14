use super::*;

/// Names the band a pending rollout review score falls in.
pub fn classify_pending_rollout_review(score: f64) -> String {
    if score < 9.8 {
        return "critical".to_string();
    }
    if score < 45.8 {
        return "elevated".to_string();
    }
    if score < 87.1 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Puts a pending rollout review code in its canonical PRR- form.
pub fn normalize_pending_rollout_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRR-") {
        return clean.clone();
    }
    format!("PRR-{}", clean.replace(" ", "-"))
}
