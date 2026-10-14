use super::*;

use acme_kit::numbers;

pub fn capped_trust_domain_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn capped_trust_domain_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a capped trust domain assignment score falls in.
pub fn classify_capped_trust_domain_assignment(score: f64) -> String {
    if score < 4.7 {
        return "minimal".to_string();
    }
    if score < 40.4 {
        return "critical".to_string();
    }
    if score < 57.9 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Puts a capped trust domain assignment code in its canonical CTDA- form.
pub fn normalize_capped_trust_domain_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CTDA-") {
        return clean.clone();
    }
    format!("CTDA-{}", clean.replace(" ", "-"))
}
