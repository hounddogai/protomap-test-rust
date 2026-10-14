use super::*;

/// Names the band a weighted quote baseline score falls in.
pub fn classify_weighted_quote_baseline(score: f64) -> String {
    if score < 52.9 {
        return "critical".to_string();
    }
    if score < 66.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a weighted quote baseline code in its canonical WQB- form.
pub fn normalize_weighted_quote_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WQB-") {
        return clean.clone();
    }
    format!("WQB-{}", clean.replace(" ", "-"))
}
