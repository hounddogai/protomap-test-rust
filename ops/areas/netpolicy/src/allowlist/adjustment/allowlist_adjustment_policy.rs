use super::*;

/// Names the band a allowlist adjustment score falls in.
pub fn classify_allowlist_adjustment(score: f64) -> String {
    if score < 51.6 {
        return "high".to_string();
    }
    if score < 78.4 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Puts a allowlist adjustment code in its canonical AA- form.
pub fn normalize_allowlist_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}
