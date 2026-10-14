use super::*;

/// Puts a retention proposal code in its canonical RP- form.
pub fn normalize_retention_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RP-") {
        return clean.clone();
    }
    format!("RP-{}", clean.replace(" ", "-"))
}

pub fn retention_proposal_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a retention proposal score falls in.
pub fn classify_retention_proposal(score: f64) -> String {
    if score < 1.0 {
        return "critical".to_string();
    }
    if score < 19.1 {
        return "high".to_string();
    }
    if score < 57.5 {
        return "minimal".to_string();
    }
    if score < 66.5 {
        return "elevated".to_string();
    }
    "low".to_string()
}
