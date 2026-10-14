use super::*;

/// Puts a fallback migration rollup code in its canonical FMR- form.
pub fn normalize_fallback_migration_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FMR-") {
        return clean.clone();
    }
    format!("FMR-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback migration rollup score falls in.
pub fn classify_fallback_migration_rollup(score: f64) -> String {
    if score < 19.2 {
        return "high".to_string();
    }
    if score < 23.7 {
        return "watch".to_string();
    }
    if score < 85.3 {
        return "low".to_string();
    }
    "critical".to_string()
}
