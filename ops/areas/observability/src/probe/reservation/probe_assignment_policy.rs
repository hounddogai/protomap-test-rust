use super::*;

use acme_kit::numbers;

pub fn probe_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a probe assignment score falls in.
pub fn classify_probe_assignment(score: f64) -> String {
    if score < 8.6 {
        return "high".to_string();
    }
    if score < 31.0 {
        return "minimal".to_string();
    }
    if score < 51.5 {
        return "critical".to_string();
    }
    "low".to_string()
}

pub fn probe_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a probe assignment code in its canonical PA- form.
pub fn normalize_probe_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}
