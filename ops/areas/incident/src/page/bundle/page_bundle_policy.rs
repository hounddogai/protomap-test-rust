use super::*;

use acme_kit::numbers;

pub fn page_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a page bundle score falls in.
pub fn classify_page_bundle(score: f64) -> String {
    if score < 46.9 {
        return "high".to_string();
    }
    if score < 82.4 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Puts a page bundle code in its canonical PB- form.
pub fn normalize_page_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PB-") {
        return clean.clone();
    }
    format!("PB-{}", clean.replace(" ", "-"))
}
