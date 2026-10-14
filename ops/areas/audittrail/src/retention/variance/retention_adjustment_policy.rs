use super::*;

/// Names the band a retention adjustment score falls in.
pub fn classify_retention_adjustment(score: f64) -> String {
    if score < 18.3 {
        return "critical".to_string();
    }
    if score < 35.5 {
        return "elevated".to_string();
    }
    if score < 89.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Puts a retention adjustment code in its canonical RA- form.
pub fn normalize_retention_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}
