use super::*;

/// Names the band a manual evidence draft score falls in.
pub fn classify_manual_evidence_draft(score: f64) -> String {
    if score < 22.9 {
        return "ok".to_string();
    }
    if score < 47.7 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a manual evidence draft code in its canonical MED- form.
pub fn normalize_manual_evidence_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MED-") {
        return clean.clone();
    }
    format!("MED-{}", clean.replace(" ", "-"))
}
