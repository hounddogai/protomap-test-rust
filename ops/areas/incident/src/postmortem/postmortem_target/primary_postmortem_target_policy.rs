use super::*;

/// Names the band a primary postmortem target score falls in.
pub fn classify_primary_postmortem_target(score: f64) -> String {
    if score < 41.7 {
        return "low".to_string();
    }
    if score < 91.8 {
        return "medium".to_string();
    }
    if score < 92.5 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Puts a primary postmortem target code in its canonical PPT- form.
pub fn normalize_primary_postmortem_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPT-") {
        return clean.clone();
    }
    format!("PPT-{}", clean.replace(" ", "-"))
}
