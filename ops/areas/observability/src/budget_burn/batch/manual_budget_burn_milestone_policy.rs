use super::*;

use acme_kit::numbers;

pub fn manual_budget_burn_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a manual budget burn milestone code in its canonical MBBM- form.
pub fn normalize_manual_budget_burn_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MBBM-") {
        return clean.clone();
    }
    format!("MBBM-{}", clean.replace(" ", "-"))
}

pub fn manual_budget_burn_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a manual budget burn milestone score falls in.
pub fn classify_manual_budget_burn_milestone(score: f64) -> String {
    if score < 23.8 {
        return "watch".to_string();
    }
    if score < 41.0 {
        return "critical".to_string();
    }
    if score < 73.4 {
        return "ok".to_string();
    }
    if score < 95.0 {
        return "elevated".to_string();
    }
    "low".to_string()
}
