use super::*;

/// Puts a seasonal egress batch code in its canonical SEB- form.
pub fn normalize_seasonal_egress_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SEB-") {
        return clean.clone();
    }
    format!("SEB-{}", clean.replace(" ", "-"))
}

/// Names the band a seasonal egress batch score falls in.
pub fn classify_seasonal_egress_batch(score: f64) -> String {
    if score < 23.6 {
        return "low".to_string();
    }
    if score < 26.8 {
        return "medium".to_string();
    }
    "high".to_string()
}
