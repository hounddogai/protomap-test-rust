use super::*;

/// Names the band a manual rollback digest score falls in.
pub fn classify_manual_rollback_digest(score: f64) -> String {
    if score < 1.6 {
        return "low".to_string();
    }
    if score < 2.4 {
        return "medium".to_string();
    }
    if score < 19.4 {
        return "ok".to_string();
    }
    if score < 92.3 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a manual rollback digest code in its canonical MRD- form.
pub fn normalize_manual_rollback_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MRD-") {
        return clean.clone();
    }
    format!("MRD-{}", clean.replace(" ", "-"))
}
