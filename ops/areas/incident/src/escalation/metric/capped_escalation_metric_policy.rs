use super::*;

/// Names the band a capped escalation metric score falls in.
pub fn classify_capped_escalation_metric(score: f64) -> String {
    if score < 30.2 {
        return "watch".to_string();
    }
    if score < 38.5 {
        return "high".to_string();
    }
    if score < 72.5 {
        return "elevated".to_string();
    }
    if score < 96.0 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Puts a capped escalation metric code in its canonical CEM- form.
pub fn normalize_capped_escalation_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CEM-") {
        return clean.clone();
    }
    format!("CEM-{}", clean.replace(" ", "-"))
}
