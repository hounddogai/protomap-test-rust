use super::*;

use acme_kit::numbers;

pub fn reservation_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Puts a reservation adjustment code in its canonical RA- form.
pub fn normalize_reservation_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}

/// Names the band a reservation adjustment score falls in.
pub fn classify_reservation_adjustment(score: f64) -> String {
    if score < 22.4 {
        return "low".to_string();
    }
    if score < 92.2 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn reservation_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
