use super::*;

/// Puts a probe entry code in its canonical PE- form.
pub fn normalize_probe_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PE-") {
        return clean.clone();
    }
    format!("PE-{}", clean.replace(" ", "-"))
}

/// Names the band a probe entry score falls in.
pub fn classify_probe_entry(score: f64) -> String {
    if score < 13.5 {
        return "high".to_string();
    }
    if score < 88.7 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
