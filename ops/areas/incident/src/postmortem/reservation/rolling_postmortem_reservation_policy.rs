use super::*;

/// Names the band a rolling postmortem reservation score falls in.
pub fn classify_rolling_postmortem_reservation(score: f64) -> String {
    if score < 37.0 {
        return "medium".to_string();
    }
    if score < 66.0 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Puts a rolling postmortem reservation code in its canonical RPR- form.
pub fn normalize_rolling_postmortem_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPR-") {
        return clean.clone();
    }
    format!("RPR-{}", clean.replace(" ", "-"))
}
