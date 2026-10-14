use super::*;

/// Names the band a reservation draft score falls in.
pub fn classify_reservation_draft(score: f64) -> String {
    if score < 36.9 {
        return "critical".to_string();
    }
    if score < 54.6 {
        return "high".to_string();
    }
    if score < 84.3 {
        return "ok".to_string();
    }
    if score < 90.5 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Puts a reservation draft code in its canonical RD- form.
pub fn normalize_reservation_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RD-") {
        return clean.clone();
    }
    format!("RD-{}", clean.replace(" ", "-"))
}
