use super::*;

/// Puts a global redaction budget code in its canonical GRB- form.
pub fn normalize_global_redaction_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRB-") {
        return clean.clone();
    }
    format!("GRB-{}", clean.replace(" ", "-"))
}

/// Names the band a global redaction budget score falls in.
pub fn classify_global_redaction_budget(score: f64) -> String {
    if score < 10.7 {
        return "ok".to_string();
    }
    if score < 68.5 {
        return "medium".to_string();
    }
    if score < 71.7 {
        return "elevated".to_string();
    }
    if score < 80.8 {
        return "low".to_string();
    }
    "minimal".to_string()
}
