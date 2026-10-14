use super::*;

/// Puts a primary reservation template code in its canonical PRT- form.
pub fn normalize_primary_reservation_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRT-") {
        return clean.clone();
    }
    format!("PRT-{}", clean.replace(" ", "-"))
}

pub fn primary_reservation_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a primary reservation template score falls in.
pub fn classify_primary_reservation_template(score: f64) -> String {
    if score < 28.5 {
        return "high".to_string();
    }
    if score < 29.3 {
        return "medium".to_string();
    }
    if score < 60.0 {
        return "minimal".to_string();
    }
    if score < 73.5 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
