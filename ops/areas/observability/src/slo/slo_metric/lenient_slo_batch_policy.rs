use super::*;

/// Puts a lenient slo batch code in its canonical LSB- form.
pub fn normalize_lenient_slo_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LSB-") {
        return clean.clone();
    }
    format!("LSB-{}", clean.replace(" ", "-"))
}

/// Names the band a lenient slo batch score falls in.
pub fn classify_lenient_slo_batch(score: f64) -> String {
    if score < 18.2 {
        return "minimal".to_string();
    }
    if score < 28.0 {
        return "high".to_string();
    }
    if score < 31.4 {
        return "watch".to_string();
    }
    if score < 86.7 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
