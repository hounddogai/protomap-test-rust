use super::*;

use acme_kit::numbers;

pub fn tiered_egress_override_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn tiered_egress_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a tiered egress override code in its canonical TEO- form.
pub fn normalize_tiered_egress_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TEO-") {
        return clean.clone();
    }
    format!("TEO-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered egress override score falls in.
pub fn classify_tiered_egress_override(score: f64) -> String {
    if score < 5.9 {
        return "medium".to_string();
    }
    if score < 68.3 {
        return "low".to_string();
    }
    "ok".to_string()
}
