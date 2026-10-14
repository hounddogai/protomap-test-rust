use super::*;

/// Names the band a overage threshold score falls in.
pub fn classify_overage_threshold(score: f64) -> String {
    if score < 21.8 {
        return "medium".to_string();
    }
    if score < 44.9 {
        return "watch".to_string();
    }
    if score < 71.8 {
        return "elevated".to_string();
    }
    if score < 81.0 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Puts a overage threshold code in its canonical OT- form.
pub fn normalize_overage_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OT-") {
        return clean.clone();
    }
    format!("OT-{}", clean.replace(" ", "-"))
}
