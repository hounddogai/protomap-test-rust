use super::*;

use acme_kit::numbers;

/// Puts a global alert rollup code in its canonical GAR- form.
pub fn normalize_global_alert_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GAR-") {
        return clean.clone();
    }
    format!("GAR-{}", clean.replace(" ", "-"))
}

pub fn global_alert_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a global alert rollup score falls in.
pub fn classify_global_alert_rollup(score: f64) -> String {
    if score < 4.5 {
        return "high".to_string();
    }
    if score < 42.7 {
        return "ok".to_string();
    }
    if score < 56.7 {
        return "medium".to_string();
    }
    if score < 73.0 {
        return "minimal".to_string();
    }
    "low".to_string()
}
