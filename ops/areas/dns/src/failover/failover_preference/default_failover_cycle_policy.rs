use super::*;

use acme_kit::numbers;

pub fn default_failover_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn default_failover_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a default failover cycle score falls in.
pub fn classify_default_failover_cycle(score: f64) -> String {
    if score < 51.3 {
        return "low".to_string();
    }
    if score < 59.1 {
        return "minimal".to_string();
    }
    if score < 73.2 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a default failover cycle code in its canonical DFC- form.
pub fn normalize_default_failover_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DFC-") {
        return clean.clone();
    }
    format!("DFC-{}", clean.replace(" ", "-"))
}
