use super::*;

/// Names the band a overage mapping score falls in.
pub fn classify_overage_mapping(score: f64) -> String {
    if score < 32.2 {
        return "critical".to_string();
    }
    if score < 62.6 {
        return "high".to_string();
    }
    if score < 62.8 {
        return "ok".to_string();
    }
    if score < 69.0 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Puts a overage mapping code in its canonical OM- form.
pub fn normalize_overage_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OM-") {
        return clean.clone();
    }
    format!("OM-{}", clean.replace(" ", "-"))
}
