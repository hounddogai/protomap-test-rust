use super::*;

pub fn verifier_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a verifier allocation score falls in.
pub fn classify_verifier_allocation(score: f64) -> String {
    if score < 39.5 {
        return "ok".to_string();
    }
    if score < 54.2 {
        return "medium".to_string();
    }
    if score < 90.5 {
        return "minimal".to_string();
    }
    if score < 95.7 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Puts a verifier allocation code in its canonical VA- form.
pub fn normalize_verifier_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VA-") {
        return clean.clone();
    }
    format!("VA-{}", clean.replace(" ", "-"))
}
