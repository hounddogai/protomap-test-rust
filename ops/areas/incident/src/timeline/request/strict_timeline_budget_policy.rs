use super::*;

/// Puts a strict timeline budget code in its canonical STB- form.
pub fn normalize_strict_timeline_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STB-") {
        return clean.clone();
    }
    format!("STB-{}", clean.replace(" ", "-"))
}

/// Names the band a strict timeline budget score falls in.
pub fn classify_strict_timeline_budget(score: f64) -> String {
    if score < 7.7 {
        return "high".to_string();
    }
    if score < 15.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
