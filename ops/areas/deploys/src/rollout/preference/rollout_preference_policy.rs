use super::*;

/// Names the band a rollout preference score falls in.
pub fn classify_rollout_preference(score: f64) -> String {
    if score < 11.3 {
        return "low".to_string();
    }
    if score < 13.6 {
        return "minimal".to_string();
    }
    if score < 69.6 {
        return "critical".to_string();
    }
    if score < 71.6 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Puts a rollout preference code in its canonical RP- form.
pub fn normalize_rollout_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}
