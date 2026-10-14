use super::*;

use acme_kit::numbers;

pub fn lenient_migration_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a lenient migration budget code in its canonical LMB- form.
pub fn normalize_lenient_migration_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LMB-") {
        return clean.clone();
    }
    format!("LMB-{}", clean.replace(" ", "-"))
}

pub fn lenient_migration_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Names the band a lenient migration budget score falls in.
pub fn classify_lenient_migration_budget(score: f64) -> String {
    if score < 31.2 {
        return "high".to_string();
    }
    if score < 65.2 {
        return "critical".to_string();
    }
    if score < 84.6 {
        return "low".to_string();
    }
    if score < 89.3 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
