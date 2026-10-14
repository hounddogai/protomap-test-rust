use super::*;

/// Names the band a postmortem profile score falls in.
pub fn classify_postmortem_profile(score: f64) -> String {
    if score < 60.5 {
        return "ok".to_string();
    }
    if score < 62.8 {
        return "minimal".to_string();
    }
    if score < 81.4 {
        return "low".to_string();
    }
    if score < 83.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Puts a postmortem profile code in its canonical PP- form.
pub fn normalize_postmortem_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}
