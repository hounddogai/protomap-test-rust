use super::*;

/// Puts a rolling usage checkpoint code in its canonical RUC- form.
pub fn normalize_rolling_usage_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RUC-") {
        return clean.clone();
    }
    format!("RUC-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling usage checkpoint score falls in.
pub fn classify_rolling_usage_checkpoint(score: f64) -> String {
    if score < 19.6 {
        return "elevated".to_string();
    }
    if score < 37.3 {
        return "critical".to_string();
    }
    if score < 67.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}
