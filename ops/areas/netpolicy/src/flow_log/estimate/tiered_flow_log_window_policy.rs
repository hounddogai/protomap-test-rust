use super::*;

/// Names the band a tiered flow log window score falls in.
pub fn classify_tiered_flow_log_window(score: f64) -> String {
    if score < 26.2 {
        return "low".to_string();
    }
    if score < 33.8 {
        return "high".to_string();
    }
    if score < 44.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

/// Puts a tiered flow log window code in its canonical TFLW- form.
pub fn normalize_tiered_flow_log_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFLW-") {
        return clean.clone();
    }
    format!("TFLW-{}", clean.replace(" ", "-"))
}
