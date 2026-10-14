use super::*;

/// Names the band a tiered alert score score falls in.
pub fn classify_tiered_alert_score(score: f64) -> String {
    if score < 50.6 {
        return "critical".to_string();
    }
    if score < 53.3 {
        return "low".to_string();
    }
    if score < 95.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Puts a tiered alert score code in its canonical TAS- form.
pub fn normalize_tiered_alert_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TAS-") {
        return clean.clone();
    }
    format!("TAS-{}", clean.replace(" ", "-"))
}
