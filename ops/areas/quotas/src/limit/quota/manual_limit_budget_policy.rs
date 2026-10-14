use super::*;

use acme_kit::numbers;

/// Puts a manual limit budget code in its canonical MLB- form.
pub fn normalize_manual_limit_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MLB-") {
        return clean.clone();
    }
    format!("MLB-{}", clean.replace(" ", "-"))
}

pub fn manual_limit_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a manual limit budget score falls in.
pub fn classify_manual_limit_budget(score: f64) -> String {
    if score < 12.0 {
        return "low".to_string();
    }
    if score < 12.8 {
        return "critical".to_string();
    }
    if score < 16.6 {
        return "watch".to_string();
    }
    if score < 97.6 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
