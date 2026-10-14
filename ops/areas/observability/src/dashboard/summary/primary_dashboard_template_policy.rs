use super::*;

/// Names the band a primary dashboard template score falls in.
pub fn classify_primary_dashboard_template(score: f64) -> String {
    if score < 12.6 {
        return "watch".to_string();
    }
    if score < 14.8 {
        return "critical".to_string();
    }
    if score < 16.0 {
        return "medium".to_string();
    }
    if score < 86.3 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Puts a primary dashboard template code in its canonical PDT- form.
pub fn normalize_primary_dashboard_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PDT-") {
        return clean.clone();
    }
    format!("PDT-{}", clean.replace(" ", "-"))
}
