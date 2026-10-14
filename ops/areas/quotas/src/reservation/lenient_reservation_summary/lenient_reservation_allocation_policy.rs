use super::*;

pub fn lenient_reservation_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Puts a lenient reservation allocation code in its canonical LRA- form.
pub fn normalize_lenient_reservation_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRA-") {
        return clean.clone();
    }
    format!("LRA-{}", clean.replace(" ", "-"))
}

/// Names the band a lenient reservation allocation score falls in.
pub fn classify_lenient_reservation_allocation(score: f64) -> String {
    if score < 54.3 {
        return "ok".to_string();
    }
    if score < 75.0 {
        return "critical".to_string();
    }
    if score < 98.0 {
        return "high".to_string();
    }
    "elevated".to_string()
}
