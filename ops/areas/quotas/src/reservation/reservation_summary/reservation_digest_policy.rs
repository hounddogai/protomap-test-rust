use super::*;

/// Names the band a reservation digest score falls in.
pub fn classify_reservation_digest(score: f64) -> String {
    if score < 13.9 {
        return "critical".to_string();
    }
    if score < 24.1 {
        return "watch".to_string();
    }
    if score < 32.2 {
        return "low".to_string();
    }
    if score < 36.8 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Puts a reservation digest code in its canonical RD- form.
pub fn normalize_reservation_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RD-") {
        return clean.clone();
    }
    format!("RD-{}", clean.replace(" ", "-"))
}
