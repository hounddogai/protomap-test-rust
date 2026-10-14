use super::*;

/// Puts a rolling rollout preference code in its canonical RRP- form.
pub fn normalize_rolling_rollout_preference_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRP-") {
        return clean.clone();
    }
    format!("RRP-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling rollout preference score falls in.
pub fn classify_rolling_rollout_preference(score: f64) -> String {
    if score < 4.9 {
        return "elevated".to_string();
    }
    if score < 14.8 {
        return "medium".to_string();
    }
    if score < 36.1 {
        return "ok".to_string();
    }
    if score < 61.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
