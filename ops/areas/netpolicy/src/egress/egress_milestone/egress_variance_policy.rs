use super::*;

/// Puts a egress variance code in its canonical EV- form.
pub fn normalize_egress_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EV-") {
        return clean.clone();
    }
    format!("EV-{}", clean.replace(" ", "-"))
}

/// Names the band a egress variance score falls in.
pub fn classify_egress_variance(score: f64) -> String {
    if score < 55.6 {
        return "low".to_string();
    }
    if score < 63.3 {
        return "minimal".to_string();
    }
    if score < 91.4 {
        return "critical".to_string();
    }
    "watch".to_string()
}
