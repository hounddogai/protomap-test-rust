use super::*;

use acme_kit::numbers;

pub fn flow_log_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a flow log batch code in its canonical FLB- form.
pub fn normalize_flow_log_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLB-") {
        return clean.clone();
    }
    format!("FLB-{}", clean.replace(" ", "-"))
}
