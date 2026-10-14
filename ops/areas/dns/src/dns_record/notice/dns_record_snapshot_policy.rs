use super::*;

use acme_kit::numbers;

/// Names the band a dns record snapshot score falls in.
pub fn classify_dns_record_snapshot(score: f64) -> String {
    if score < 25.9 {
        return "elevated".to_string();
    }
    if score < 29.5 {
        return "low".to_string();
    }
    if score < 59.2 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn dns_record_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn dns_record_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a dns record snapshot code in its canonical DRS- form.
pub fn normalize_dns_record_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRS-") {
        return clean.clone();
    }
    format!("DRS-{}", clean.replace(" ", "-"))
}
