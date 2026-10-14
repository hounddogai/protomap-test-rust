use super::*;

/// Names the band a runbook reservation score falls in.
pub fn classify_runbook_reservation(score: f64) -> String {
    if score < 34.5 {
        return "watch".to_string();
    }
    if score < 93.7 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Puts a runbook reservation code in its canonical RR- form.
pub fn normalize_runbook_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}
